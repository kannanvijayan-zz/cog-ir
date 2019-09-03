
extern crate cog_ir;

#[macro_use]
extern crate log;
extern crate env_logger;

use cog_ir::api::{ build, graph, Int32Ty };

fn main() {
    env_logger::builder()
        .default_format_timestamp(false)
        .default_format_module_path(false)
        .init();

    // Build the following graph:
    //
    //                    +-----------+
    //                    | @start    |
    //                    | a = 0     |
    //                    | b = 10    |
    //                    | c = a==b  |
    //                    | branch c  |
    //                    +-----------+
    //                          v
    //                          |
    //          +-------------------+
    //          |false              |true
    //          |                   |
    //          |                   |
    //          v                   v           Subgraph
    //      +-----------+     +-----------+
    //      | @a        |     | @b        |     
    //      | d = a+1   |     | e = a+b   |
    //      | jump      |     | f = 9     |
    //      +-----------+     | g = e==f  |
    //        |               | branch g  |
    //        |               +-----------+
    //        |                   |
    //        |   +---------------+-----------+ Subgraph
    //        |   |                . . . . . .|. . . . .
    //        v   v                .          v        .
    //  +-----------+              .    +-----------+  .
    //  | @c        |              .    | @d        |  .
    //  | h = [d|f] | <--+         .    | i = f+1   |  .
    //  | jump      |    |         .    | jump      |  .
    //  +-----------+    |         .    +-----------+  .
    //          |        |         . . . . .|. . . . . .
    //          |        |                  |
    //          +--------+------+-----------+
    //                          |
    //                          v
    //                  +-----------+
    //                  | @e        |
    //                  | j = [h|i] |
    //                  | ret j     |
    //                  +-----------+
    //
    //
    let builder = build(|bs| {
        info!("Building graph.");

        let block_a = bs.decl_plain_block(0);
        let block_b = bs.decl_plain_block(0);
        let block_c = bs.decl_plain_block(1);
        let block_e = bs.decl_plain_block(1);

        let a = bs.emit_const_int32(0);
        let b = bs.emit_const_int32(10);
        let c = bs.emit_eq(a, b);
        bs.branch(c, block_a, &[], block_b, &[]);
        info!("a={:?}, b={:?}, c={:?}",
              a.as_u32(), b.as_u32(), c.as_u32());

        bs.def_block(block_a);
        let t0_const1 = bs.emit_const_int32(1);
        let d = bs.emit_add(a, t0_const1);
        bs.jump(block_c, &[d.untyped_defn()]);

        bs.def_block(block_b);
        bs.def_subgraph(|bs| {
            let block_d = bs.decl_plain_block(0);
            let e = bs.emit_add(a, b);
            let f = bs.emit_const_int32(9);
            let g = bs.emit_eq(e, f);
            bs.branch(g, block_c, &[f.untyped_defn()],
                         block_d, &[]);

            bs.def_block(block_d);
            let t1_const1 = bs.emit_const_int32(1);
            let i = bs.emit_add(f, t1_const1);
            bs.jump(block_e, &[i.untyped_defn()]);
        });

        bs.def_block(block_c);
        let h = bs.emit_phi::<Int32Ty>();
        bs.jump(block_e, &[h.untyped_defn()]);

        bs.def_block(block_e);
        let j = bs.emit_phi::<Int32Ty>();
        bs.ret(j);
    });

    builder.dump_stats("test1(simple subgraph)");

    graph(builder, |gs| {
        gs.debug_print_cur_instr();
        let mut i = 0_u32;
        loop {
            let defn = match gs.next_defn() {
              Some(defn) => defn, None => { break; }
            };
            debug!("i0: {}", defn);
            gs.debug_print_cur_instr();
        }
    });
}
