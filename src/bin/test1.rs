
extern crate cog_ir;

#[macro_use]
extern crate log;
extern crate env_logger;

use cog_ir::api::{ build, Int32Ty };

fn main() {
    env_logger::builder()
        .default_format_timestamp(false)
        .default_format_module_path(false)
        .init();

    // Build the following graph:
    //
    //                    +---------------+
    //                    | @start        |
    //                    | a = 0         |
    //                    | b = 10        |
    //                    | jump A        |
    //                    |  0:a          |
    //                    +---------------+
    //                            v
    //                            |
    //   +-------------------+    |
    //   |                   |    |
    //   |                   v    v
    //   |                +---------------+
    //   |                | @A            |     
    //   |                | c = phi0      |
    //   |                | d = c < b     |
    //   |                | branch B,C    |
    //   |                +---------------+
    //   |                      v   v
    //   |                true  |   | false
    //   |         +------------+   |
    //   |         |                |
    //   |         v                |
    //   |   +---------------+      |
    //   |   | @B            |      |
    //   |   | e = c+1       |      |
    //   |   | jump A        |      |
    //   |   |  0: [e]       |      |
    //   |   +---------------+      |
    //   |        |                 |
    //   |        |                 |
    //   +--------+             +---+
    //                          |
    //                          v
    //                  +-----------+
    //                  | @C        |
    //                  | ret c     |
    //                  +-----------+
    //
    //
    let builder = build(|bs| {
        info!("Building graph.");

        // Define the loop header and the return block.
        let block_a = bs.decl_loop_head(1);
        // let block_d = bs.decl_plain_block(0);

        let a = bs.emit_const_int32(0);
        let b = bs.emit_const_int32(10);
        bs.jump(block_a, &[a.untyped_defn()]);

        bs.def_loop(block_a, |bs| {
            let block_b = bs.decl_plain_block(0);
            let block_c = bs.decl_plain_block(0);

            let c = bs.emit_phi::<Int32Ty>();
            let d = bs.emit_lt(c, b);
            bs.branch(d, block_b, &[], block_c, &[]);

            bs.def_block(block_b);
            let tmp0_const1 = bs.emit_const_int32(1);
            let e = bs.emit_add(c, tmp0_const1);
            bs.jump(block_a, &[e.untyped_defn()]);

            bs.def_block(block_c);
            bs.ret(c);
        });

        // bs.def_block(block_d);
        // bs.jump(block_a, &[a.untyped_defn()]);
    });

    builder.dump_stats("test1(simple loop)");
}
