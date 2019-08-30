
# CogIR

CogIR is a lightweight, low-overhead, low-level
intermediate layer for generating machine code.

The design goal is to explicitly acknowledge
only peephole optimization and register allocation
as expected backend banalyses.

This means that much of the facilities for
instruction and graph reordering are not necessary.

Instead, CogIR focuses on efficiency of representation
and fast logic for emitting instructions and collecting
block structure.

```
*NOTE*
Currently, this implementation is just a builder
which produces a graph representation.  Actual
machine code generation is not yet implemented.
```

## A simple single-block graph construction.

```
  let builder = cog_ir::api::build(
   |bs: &mut BuildSession<'bs>| {
    // c[0-2]: Defn<'bs, Int32Ty>
    let c0 = bs.emit_const_int32(5);
    let c1 = bs.emit_const_int32(6);
    let c2 = bs.emit_const_int32(1);

    // c3: Defn<'bs, BoolTy>
    let c3 = bs.emit_const_bool(true);

    // bs.emit_add::<Int32Ty>(l, r) -> Defn<'bs, Int32Ty>
    let t0 = bs.emit_add(c0, c2);

    // bs.emit_eq::<Int32Ty>(l, r) -> Defn<'bs, BoolTy>
    let t1 = bs.emit_eq(t0, c1);

    // bs.emit_and::<BoolTy>(l, r) -> Defn<'bs, BoolTy>
    let t2 = bs.emit_and(t1, c3);

    // End instruction `ret::<BoolTy>(v)`.
    bs.ret(t2);
   });
```

Broken down into pieces:

```
  // Build a new graph.
  let builder = cog_ir::api::build(|bs| {
```

A graph build is performed by calling the `build`
function with an `FnOnce` callback function object that
specifies the graph contents.

The `bs` object has type `&mut BuildSession<'bs>` for
a lifetime of `bs` restricted to the activation lifetime
of the callback.

The initial state of the build is with an automatically
declared and entered empty start block set as the
currently active block.

```
    let c0_i32 = bs.emit_const_int32(5);
    let c1_i32 = bs.emit_const_int32(6);
    let c2_i32 = bs.emit_const_int32(1);
    let c3_bool = bs.emit_const_bool(true);
```

These calls emit the corresponding const instructions.
Each `emit_const_int32` call returns values of type
`Defn<'a, Int23Ty>`, and the `emit_const_bool` call
returns values of type `Defn<'a, BoolTy>`.

```
    let t0_i32 = bs.emit_add(c0_i32, c2_i32);
```

Here, and add instruction is emitted.  The `emit_add`
method is generic on `<T: IrType>`, which is inferred
to be `T=Int32Ty` from the inputs `c0_i32` and `c2_i32`.

It takes 2 definitions, encodes the add instruction
and its operands, and returns the definition for the
result.

As the `emit_add` method's output type is derived
from its input type, and is defined for all of
`BoolTy`, `Int32Ty`, `Int64Ty`, and `PtrIntTy`.

```
    let t1_bool = bs.emit_eq(t0_i32, c1_i32);
```

As with the add instruction, the `emit_eq` is generalized
across ir types, in this case `emit_eq::<Int32Ty>(...)`.

The result type is boolean for all inputs, so `t1_bool`
type `Defn<'bs, BoolTy>`.

```
    // Bitwise and with c3.
    let t2_bool = bs.emit_and(t1_bool, c3_bool);
```

As with the add and eq instructions, the `emit_and`
instruction is defined on all integer input types,
including bool, and returns a result of the same
type as its inputs.

```
    // Return the result.
    bs.ret(t2_bool);
```

The ret instruction is once again generic over its
input type, and is also an end instruction.  This
causes the current block to be marked finished.

```
  });
```

## Declaring blocks

Blocks must be declared before they are defined (filled
with instructions), and the number of phis in the block
must be specified at declaration time.

Blocks must be entered (to be filled) and their
instructions emitted in the order of their declaration
within the subgraph.

```
    // Three types of blocks to declare.
    bs.decl_plain_block(...);
    bs.decl_start_block(...);
    bs.decl_loop_head(...);

    // Later:
    // enter blocks for definition in various ways.

    // Enter a start or plain block:
    bs.def_block(block_a);
    bs.emit_instr_1();
    bs.emit_instr_2();
    bs.jump(...); // End instr finishes block.

    // Enter the next block in declaration order:
    bs.def_block(block_b);
    bs.emit_instr_1();
    bs.emit_instr_2();
    bs.branch(...); // End instr finishes block.
    
```

Once a block is entered, all the instructions for it
must be emitted in sequence until the terminal instruction
at which point it becomes finished.

When a block is finished, the next block (in declaration 
order) must be entered and specified.

Branches to blocks can only be forward within the
declared blocks: only blocks that have not already
been entered may be used as control flow targets.

This forces an RPO ordering to the entering of declared
blocks:

### Example (declaring blocks)
```
  let builder = build(|bs| {
    // Already entered start block.
    // Build a small diamond graph.
    //   start --> (a|b) --> c.

    // `decl_plain_block` declares a new block,
    // taking the number of phis as an argument.
    // The `c` block will choose some value from `a`
    // or `b`, so will take a phi.
    //
    // The type of `block_*` is `BlockRef<'bs>`.
    let block_a = bs.decl_plain_block(0);
    let block_b = bs.decl_plain_block(0);
    let block_c = bs.decl_plain_block(1);

    // These instructions are emitted in the
    // start block.  The logic is:
    //   c = (0_i32 == 10_i32)
    let a = bs.emit_const_int32(0);
    let b = bs.emit_const_int32(10);
    let c = bs.emit_eq(a, b);

    // The branch instruction ends the block,
    // taking 2 target blocks, as well as
    // the phi-input definitions to pass to
    // them.
    bs.branch(c, block_a, &[], block_b, &[]);

    // The `A` block can now be defined, as
    // the start block has finished.
    bs.def_block(block_a);

    // In block `A`, we compute `d = a + 1`
    let t0_const1 = bs.emit_const_int32(1);
    let d = bs.emit_add(a, t0_const1);

    // And jump to block `C` with the result,
    // passing `d` as the only phi argument
    // for the target block.
    bs.jump(block_c, &[d.untyped_defn()]);

    // That finished block `A`, and the next
    // block to be defined is `B`.
    bs.def_block(block_b);

    // Contents of `B` are `e = b - 2`.
    let t1_const2 = bs.emit_const_int32(2);
    let e = bs.emit_sub(b, t1_const2);

    // Jump to block `C` passing `e` as the phi-arg.
    bs.jump(block_c, &[e.untyped_defn()]);

    // Now we can define block `C`.
    bs.def_block(block_c);

    // Emit the 1 phi for this block.
    // The input definitions are specified
    // in the end instructions of the predecessor
    // blocks.
    let phi_0 = bs.emit_phi::<Int32Ty>();

    // Return the phi-ed value.
    bs.ret(phi_0);
  });
```

## Subgraphs

See the `src/bin/test0.rs` file for an example
of subgraph builder usage.

Forcing all blocks in the final graph to be declared
and entered in RPO is too cumbersome for complex graphs
where decisions may be made about control flow within
compiler sub-logic that is examining site-specific
data dynamically.

Nested subgraphs may be constructed arbitrarily -
at any point during graph construction, with the
following api:

```
    build(|bs| {
        ...
        bs.def_subgraph(|cs| {
            // cs: &mut BuildSession<'cs> where 'bs: 'cs
            // Declare and define own blocks here.
        });
        ...
    }
```


Nested subgraphs declare and specify their own list
of blocks (in internal RPO order), and may invoke
their own nested subgraph generators.

A nested subgraph may use definitions from its parent
graph (but care must be taken to use only definitions
from dominating blocks - this is not validated at
construction time).

A nested subgraph may also emit jumps to blocks declared
in parent subgraph builders, as long as those blocks have
not been entered (forward jumps).

When a nested subgraph is first created, the current
active block (in its current state - either entered or
finished) is borrowed, and a declaration sequence
established.  By the time the nested subgraph builder
completes, it must have specified all the blocks it
declared.

This allows parent subgraph constructors to invoke
nested subgraph constructors, passing along definitions
as compile-time arguments, and block-targets as
continuation arguments for control flow.

## Loops

See the `src/bin/test1.rs` file for an example
of building loops.

Loops are specified as sub-graphs.  A loop header block
must be declared specifically (with `decl_loop_head`),
and cannot be entered with `def_block()`.  Instead,
the `def_loop()` method must be called for the loop
block, passing in a subgraph constructor callback.

The subgraph (and any nested subgraphs) built by this
constructor are allowed to make back-references to
the loop block (or any other outer loop block).

Once the loop subgraph callback completes, the loop
header block is marked as closed, and new back-references
may no longer be made to it.

```
    build(|bs| {
        ...
        // Declare a loop block.
        let bl_loop = bs.decl_loop_head(...);

        ...
        // Later
        bs.def_loop(bl_loop, |bs| {
            // Nested BuildSession used for loop subgraph.

            // Within this builder and any nested
            // subgraphs, valid to use `bl_loop` as
            // a jump target.
        });
        // bl_loop is now closed and cannot be
        // used as a control flow target.
        ...
    }
```

## Overview

Overall, graph construction proceeds as follows:

```

 Top           Build                                End
 Subgraph:   *------->         *------->   *---- ... -->
                     /          \     /     \
                    /            \   *------->
                   /              \       Nested
                  / Nested         \      Subgraph
                 /    Subgraph      \
                /                    \
 Subgraphs     *---->  *----->  *----->
                   /    \   /    \
               Nes/ted   \ *------> Nested
             Subg/raph    \         Subgraph
                *--->  *--->
                    /   \
                   *-----> Nested
                           Subgraph
```

Each parent graph can declare blocks ahead of time,
and pass both block references and definitions from
already-emitted instructions into the the subgraph
builders.
