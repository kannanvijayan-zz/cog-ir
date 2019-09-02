
use std::mem;

use crate::block::{ Block, BlockId, BlockRef, BlockStore };
use crate::ops::{ Operation, TerminalOperation };
use crate::instr::{ InstrId, InstrStore };
use crate::defn::{ Defn, TypedDefn };

use crate::ops::{
    NopOp, PhiOp,
    ConstBoolOp, ConstInt32Op, ConstInt64Op,

    CmpOp, CmpKind,
    BiniOp, BiniKind,

    RetOp, JumpOp, BranchOp
};
use crate::ir_types::{
    IrOutputType, IrType,
    BoolTy, Int32Ty, Int64Ty,
};

pub struct Builder {
    // The instruction store.
    instr_store: InstrStore,

    // The block store.
    block_store: BlockStore,

    // The segmented completion queue for subgraph
    // block declarations, mushed together into one
    // vector.
    //
    // Subgraphs are always completed before
    // returning to the parent graph, and
    // the segments within this vector themselves are
    // tracked by subgraph_decls.
    subgraph_decls: Vec<BlockId>
}

impl Builder {
    const SUBGRAPH_DECLS_CAP: usize = 8;

    pub fn new() -> Builder {
        debug!("SizeOf(Block) = {}",
               mem::size_of::<Block>());

        let instr_store = InstrStore::new();
        let block_store = BlockStore::new();
        let subgraph_decls =
          Vec::with_capacity(Self::SUBGRAPH_DECLS_CAP);

        Builder { instr_store, block_store, subgraph_decls }
    }

    pub(crate) fn build<F>(f: F) -> Builder
        where F: for<'x> FnOnce (&mut BuildSession<'x>)
    {
        let mut builder = Builder::new();
        let start_block = builder.block_store
                                 .start_block_id();

        let mut sess =
          BuildSession::new(&mut builder,
                            BlockRef::new(start_block),
                            /* emitted_phis = */ 0);
        f(&mut sess);

        // At the end of the session, all blocks must
        // be completely specified.
        for bl_ref in builder.block_store.iter_blocks() {
            assert!(bl_ref.has_finished(),
                    "Block {} unfinished",
                    bl_ref.id().as_u32());
            if bl_ref.is_loop() {
                debug_assert!(
                  bl_ref.has_loop_complete(),
                  "Loop for block {} incomplete",
                  bl_ref.id().as_u32());
            }
        }

        builder
    }

    pub fn dump_stats(&self, name: &'static str) {
        debug!("Builder {} instrs={} blocks={}",
               name,
               self.instr_store.instr_bytes_len(),
               self.block_store.total_blocks());
    }
}

/**
 * A graph build is parameterized around the lifetime
 * of a build session.
 *
 * Each subgraph builder gets its own build session.
 * The build session keeps track of the segment of
 * the subgraph_queues dedicated to it, and the
 * specification progress along it.
 */
pub struct BuildSession<'bs> {
    builder: &'bs mut Builder,

    // The current block being built.
    cur_block: BlockRef<'bs>,

    // The number phi instructions emitted in this block
    emitted_phis: u32,

    // The start index into subgraph_blocks for start
    // of the blocks declared by this subgraph.
    //
    // Initialized to length of vector at session start.
    subgraph_start: u32,

    // The number of entered blocks in this subgraph.
    //
    // Initialized to 0 at session start.
    subgraph_entered: u32,
}

impl<'bs> BuildSession<'bs> {
    fn new(builder: &'bs mut Builder,
           cur_block: BlockRef<'bs>,
           emitted_phis: u32)
      -> BuildSession<'bs>
    {
        let subgraph_start =
          builder.subgraph_decls.len() as u32;

        BuildSession {
            builder, cur_block, emitted_phis,
            subgraph_start,
            subgraph_entered: 0
        }
    }

    // Retrieve the a reference to the actual
    // block from a `BlockRef` index.
    fn get_block(&self, block: BlockRef<'bs>) -> &Block {
        unsafe {
            self.builder.block_store.get_block(block.id())
        }
    }

    fn subgraph_decls_len(&self) -> u32 {
        self.builder.subgraph_decls.len() as u32
    }
    fn subgraph_cur_end(&self) -> u32 {
        let end = self.subgraph_start
                    + self.subgraph_entered;
        debug_assert!(end <= self.subgraph_decls_len());

        end
    }
    fn subgraph_cur_idx(&self, offset: u32) -> u32 {
        debug_assert!(offset < self.subgraph_entered);
        let idx = self.subgraph_start + offset;
        debug_assert!(idx < self.subgraph_decls_len());
        idx
    }

    // Retrieve a reference to the specific subgraph
    // block at the given index.
    fn get_subgraph_block(&self, offset: u32) -> &Block {
        let idx = self.subgraph_cur_idx(offset) as usize;
        let block_id = self.builder.subgraph_decls[idx];
        unsafe {
          self.builder.block_store.get_block(block_id)
        }
    }

    // Test if the current subgraph is complete.
    fn subgraph_complete(&self) -> bool {
        self.subgraph_cur_end()
          == self.subgraph_decls_len()
    }

    fn get_cur_block(&self) -> &Block {
        self.get_block(self.cur_block)
    }

    fn next_spec_block(&self) -> BlockRef<'bs> {
        let idx = self.subgraph_cur_end();
        debug_assert!(idx < self.subgraph_decls_len());

        let id = unsafe {
          *self.builder.subgraph_decls.get_unchecked(
            idx as usize)
        };

        BlockRef::new(id)
    }

    // Declare a new block.
    pub fn decl_plain_block(&mut self, num_phis: u32)
      -> BlockRef<'bs>
    {
        let id = self.builder.block_store
                     .decl_plain_block(num_phis);
        self.builder.subgraph_decls.push(id);
        debug!("Decl plain block phis={} id={}",
               num_phis, id.as_u32());
        BlockRef::new(id)
    }

    // Declare a start block.
    pub fn decl_start_block(&mut self)
      -> BlockRef<'bs>
    {
        let id = self.builder.block_store
                     .decl_start_block();
        self.builder.subgraph_decls.push(id);
        BlockRef::new(id)
    }

    // Declare a new loop header block.
    pub fn decl_loop_head(&mut self, num_phis: u32)
      -> BlockRef<'bs>
    {
        let id = self.builder.block_store
                     .decl_loop_head(num_phis);
        self.builder.subgraph_decls.push(id);
        BlockRef::new(id)
    }

    fn def_block_impl(&mut self, block: BlockRef<'bs>) {
        // Ensure current block is finished.
        assert!(self.get_cur_block().has_finished());

        // Ensure that the block being entered is
        // the next on the declared block list for
        // this subgraph.
        assert!(block == self.next_spec_block());

        unsafe {
            self.builder.block_store.enter_block(
              block.id(),
              self.builder.instr_store.front_instr_id());
        }

        // Update the current block, and the
        // `subgraph_entered` index.
        // after it.
        self.cur_block = block;
        self.subgraph_entered += 1;

        // Reset the emitted phis for a new block.
        self.emitted_phis = 0;
    }

    // Enter the next block.  The current block
    // must have been finished with a block-end instr.
    pub fn def_block(&mut self, block: BlockRef<'bs>) {
        // Ensure that the block being entered is
        // not a loop block.  Loops must be
        // defined with `def_loop`.
        assert!(! self.get_block(block).is_loop());

        self.def_block_impl(block);
    }

    // Enter and specify a sub-graph
    pub fn def_subgraph<'cs, R, F>(&'cs mut self, f: F)
        -> R
      where F: FnOnce (&mut BuildSession<'cs>) -> R
    {
        // The cur_block for a new session is borrowed
        // from the cur_block for the current session.
        let cur_block = self.cur_block;
        let emitted_phis = self.emitted_phis;
        let (new_block_id, r) = {
            let mut sub_sess: BuildSession<'cs> =
              BuildSession::new(
                &mut self.builder,
                cur_block,
                emitted_phis);
            let r = f(&mut sub_sess);
            sub_sess.assert_complete();

            (sub_sess.cur_block.id(), r)
        };

        // Update the cur_block of this session with
        // the sub-session's cur_block.
        //
        // TODO: Insert long explanation here for why
        // the rest of the graph construction logic
        // allows us to do this blindly.
        self.cur_block = BlockRef::new(new_block_id);

        r
    }

    fn assert_complete(&mut self) {
        // All the blocks in the array from
        // the sub-session's start index to
        // the end, must be entered.  And
        // all of them except optionally the
        // last must be finished.
        assert!(self.subgraph_complete());
        let ent = self.subgraph_entered;
        if ent > 0 {
            for i in 0 .. ent-1 {
                let bl = self.get_subgraph_block(i);
                assert!(bl.has_finished());
            }
        }
    }

    // Enter and specify a loop.
    pub fn def_loop<'cs, R, F>(&'cs mut self,
        loop_block: BlockRef<'bs>, f: F) -> R
      where F: FnOnce (&mut BuildSession<'cs>) -> R
    {
        // Ensure that the block being entered is a loop.
        assert!(self.get_block(loop_block).is_loop());

        // Start defining the loop block.
        self.def_block_impl(loop_block);

        // Immediately enter a subgraph.
        self.def_subgraph(move |cs| {
            let result = f(cs);

            // When defining a loop subgraph, the entire
            // subgraph must be complete by the return
            // point.
            //
            // Immediately after a loop subgraph definition,
            // the parent subgraph definition state is such
            // that there are no active blocks (including
            // the active loop header when the subgraph
            // was entered).
            assert!(cs.get_cur_block().has_finished());
            debug_assert!(
              cs.get_block(loop_block).has_finished());

            unsafe {
                cs.builder.block_store
                  .finish_loop(loop_block.id());
            }

            result
        })
    }

    fn emit_instr<'cs: 'bs, OP: Operation>(&mut self,
        op: OP, operands: &[Defn<'cs>])
      -> Option<TypedDefn<'bs, OP::Output>>
      where OP: Operation
    {
        assert!(! self.get_cur_block().has_finished());

        // Add the instruction to the instr store.
        let instr_id =
          self.builder.instr_store.emit_instr(
            &op, operands) ?;

        // No changes need to be made to the block store.

        Some(TypedDefn::new(instr_id))
    }

    fn emit_end<'cs: 'bs, OP>(&mut self,
        op: OP,
        operands: &[Defn<'cs>],
        targets: &[(BlockRef<'cs>, &[Defn<'cs>])])
      -> Option<InstrId>
      where OP: TerminalOperation + Operation
    {
        assert!(! self.get_cur_block().has_finished());

        // Add the instruction to the instr store.
        let instr_id =
          self.builder.instr_store.emit_end(
            &op, operands, targets) ?;

        // Mark the block as finished.
        unsafe {
            self.builder.block_store.finish_block(
              self.cur_block.id(), instr_id);
        }

        Some(instr_id)
    }

    pub fn emit_nop(&mut self) {
        self.emit_instr(NopOp::new(), &[]).unwrap();
    }
    pub fn emit_const_bool(&mut self, b: bool)
      -> TypedDefn<'bs, BoolTy>
    {
        self.emit_instr(ConstBoolOp::new(b), &[]).unwrap()
    }
    pub fn emit_const_int32(&mut self, i: u32)
      -> TypedDefn<'bs, Int32Ty>
    {
        self.emit_instr(ConstInt32Op::new(i), &[]).unwrap()
    }
    pub fn emit_const_int64(&mut self, i: u64)
      -> TypedDefn<'bs, Int64Ty>
    {
        self.emit_instr(ConstInt64Op::new(i), &[]).unwrap()
    }

    pub fn emit_cmp<'cs: 'bs, T: IrType>(&mut self,
        kind: CmpKind,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, BoolTy>
    {
        let lhs = lhs.untyped_defn();
        let rhs = rhs.untyped_defn();
        self.emit_instr(
          CmpOp::<T>::new(kind), &[lhs, rhs]).unwrap()
    }
    pub fn emit_lt<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, BoolTy>
    {
        self.emit_cmp(CmpKind::Lt, lhs, rhs)
    }
    pub fn emit_le<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, BoolTy>
    {
        self.emit_cmp(CmpKind::Le, lhs, rhs)
    }
    pub fn emit_eq<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, BoolTy>
    {
        self.emit_cmp(CmpKind::Eq, lhs, rhs)
    }
    pub fn emit_ne<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, BoolTy>
    {
        self.emit_cmp(CmpKind::Ne, lhs, rhs)
    }
    pub fn emit_ge<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, BoolTy>
    {
        self.emit_cmp(CmpKind::Ge, lhs, rhs)
    }
    pub fn emit_gt<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, BoolTy>
    {
        self.emit_cmp(CmpKind::Gt, lhs, rhs)
    }

    pub fn emit_bini<'cs: 'bs, T: IrType>(&mut self,
        kind: BiniKind,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, T>
    {
        let lhs = lhs.untyped_defn();
        let rhs = rhs.untyped_defn();
        self.emit_instr(
          BiniOp::new(kind), &[lhs, rhs]).unwrap()
    }
    pub fn emit_add<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, T>
    {
        self.emit_bini(BiniKind::Add, lhs, rhs)
    }
    pub fn emit_sub<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, T>
    {
        self.emit_bini(BiniKind::Sub, lhs, rhs)
    }
    pub fn emit_mul<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, T>
    {
        self.emit_bini(BiniKind::Mul, lhs, rhs)
    }
    pub fn emit_and<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, T>
    {
        self.emit_bini(BiniKind::And, lhs, rhs)
    }
    pub fn emit_or<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, T>
    {
        self.emit_bini(BiniKind::Or, lhs, rhs)
    }
    pub fn emit_xor<'cs: 'bs, T: IrType>(&mut self,
        lhs: TypedDefn<'cs, T>,
        rhs: TypedDefn<'cs, T>)
      -> TypedDefn<'bs, T>
    {
        self.emit_bini(BiniKind::Xor, lhs, rhs)
    }

    pub fn emit_phi<T: IrType>(&mut self)
      -> TypedDefn<'bs, T>
    {
        assert!(! self.get_cur_block().has_finished());
        debug_assert!(self.emitted_phis
                        < self.get_cur_block().num_phis());
        self.emitted_phis += 1;
        self.emit_instr(PhiOp::<T>::new(), &[]).unwrap()
    }

    pub fn ret<'cs: 'bs, T: IrType>(&mut self,
        val: TypedDefn<'cs, T>)
    {
        self.emit_end(
          RetOp::<T>::new(), &[val.untyped_defn()],
          /* targets = */ &[]).unwrap();
    }

    pub fn jump<'cs: 'bs>(&mut self,
        target: BlockRef<'cs>, phis: &[Defn<'cs>])
    {
        self.emit_end(JumpOp::new(), &[],
          /* targets = */ &[(target, phis)]).unwrap();
    }

    pub fn branch<'cs: 'bs>(&mut self,
        bit: TypedDefn<'cs, BoolTy>,

        if_true: BlockRef<'cs>,
        true_phis: &[Defn<'cs>],

        if_false: BlockRef<'cs>,
        false_phis: &[Defn<'cs>])
    {
        let bit = bit.untyped_defn();
        self.emit_end(BranchOp::new(), &[bit],
          /* targets = */ &[
            (if_true, true_phis),
            (if_false, false_phis)]).unwrap();
    }
            
}
