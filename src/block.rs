
use crate::instr::{ InstrPosn, InstrNo, Instr, EndInstr };

/**
 * A block-id identifies a block by declaration id.
 * These ids are assigned to blocks in order of global
 * declaration ordering, with the first declared
 * block having id 0.
 *
 * This ordering does not conform to RPO, as subgraph
 * builders declare their blocks after their parents,
 * but have their control flow nested within.
 */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
pub struct BlockId(u32);

impl BlockId {
    pub(crate) fn as_u32(&self) -> u32 { self.0 }
}

/**
 * The information about a block.
 */
pub struct Block {
    // The id of this block in declaration order.
    id: BlockId,

    // Some block-specific info is held inside
    // an enum helper.
    variant: BlockVariant,

    // The state of the block.
    state: BlockState,

    // The number of incoming edges to this block.
    // Incremented as edges are added.
    // For non-loop-entry blocks, this field is
    // fixed after the start of block specification.
    input_edges: u32,

    // The ordering the block in specification
    // (RPO) order.  Only set when the block is entered.
    order: u32,

    // The index of the first instruction.  Only set
    // when the block is entered.
    first_instr: InstrNo,

    // The index of the last instruction.  Only set
    // when the block is finished.
    last_instr: InstrNo,
}
enum BlockVariant {
    Plain { num_phis: u32 },
    Loop { num_phis: u32, loop_no: u16 },
    Start { start_no: u16 }
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BlockState { Declared, Entered, Finished,
                 LoopComplete }

impl Block {
    fn new(id: BlockId, variant: BlockVariant) -> Block {
        Block {
          id, variant,
          state: BlockState::Declared,
          input_edges: 0, order: u32::max_value(),
          first_instr: InstrNo::invalid(),
          last_instr: InstrNo::invalid()
        }
    }
    fn new_plain(id: BlockId, num_phis: u32) -> Block {
        Self::new(id, BlockVariant::Plain { num_phis })
    }
    fn new_start(id: BlockId, start_no: u16) -> Block {
        Self::new(id, BlockVariant::Start { start_no })
    }
    fn new_loop(id: BlockId, num_phis: u32, loop_no: u16)
      -> Block
    {
        let bv = BlockVariant::Loop { num_phis, loop_no };
        Self::new(id, bv)
    }
    pub(crate) fn id(&self) -> BlockId { self.id }

    pub fn is_start(&self) -> bool {
        match self.variant {
            BlockVariant::Start{ start_no: _ }
              => true,
            _ => false
        }
    }
    pub fn is_loop(&self) -> bool {
        match self.variant {
            BlockVariant::Loop{ num_phis: _, loop_no: _}
              => true,
            _ => false
        }
    }

    pub fn input_edges(&self) -> u32 { self.input_edges }
    pub fn has_entered(&self) -> bool {
        self.state >= BlockState::Entered
    }
    pub fn has_finished(&self) -> bool {
        self.state >= BlockState::Finished
    }
    pub fn has_loop_complete(&self) -> bool {
        self.state >= BlockState::LoopComplete
    }

    pub(crate) fn incr_input_edges(&mut self) {
        self.input_edges += 1;
    }
    fn set_entered(&mut self,
      order: u32, first_instr: InstrNo)
    {
        debug_assert!(! self.has_entered());
        self.state = BlockState::Entered;
        self.first_instr = first_instr;
        self.order = order;
    }
    fn set_add_instr(&mut self, last_instr: InstrNo) {
        debug_assert!(self.has_entered());
        debug_assert!(! self.has_finished());
        self.last_instr = last_instr;
    }
    fn set_finished(&mut self, last_instr: InstrNo) {
        debug_assert!(self.has_entered());
        debug_assert!(! self.has_finished());
        self.state = BlockState::Finished;
        self.last_instr = last_instr;
    }
    fn set_loop_complete(&mut self) {
        debug_assert!(self.has_finished());
        debug_assert!(self.is_loop());
        self.state = BlockState::LoopComplete;
    }
}

/**
 * A store of all the blocks in a graph.  A vector
 * stores all block information in global declaration
 * order, and a secondary RPO index of the first vector.
 */
pub struct BlockStore {
    instr_bytes: Vec<u8>,
    instr_posns: Vec<InstrPosn>,
    decl_blocks: Vec<Block>,
    rpo_index: Vec<BlockId>,
    cur_block_id: BlockId,
    num_starts: u16,
    num_loops: u16,
    total_phis: u32,
    num_phis: u32,
}

impl BlockStore {
    const INSTR_BYTES_CAP: usize = 256;
    const INSTR_POSNS_CAP: usize = 256;
    const DECL_BLOCKS_CAP: usize = 8;
    const RPO_INDEX_CAP: usize = 8;

    const MAX_DECL_BLOCKS: u32 = 0xff_ffff;
    const MAX_INSTRS: u32 = 0xfff_ffff;

    pub fn new() -> BlockStore {
        let instr_bytes =
          Vec::with_capacity(Self::INSTR_BYTES_CAP);
        let instr_posns =
          Vec::with_capacity(Self::INSTR_POSNS_CAP);
        let decl_blocks =
          Vec::with_capacity(Self::DECL_BLOCKS_CAP);
        let rpo_index =
          Vec::with_capacity(Self::RPO_INDEX_CAP);

        let cur_block_id = BlockId(0);

        let mut bs = BlockStore {
            instr_bytes, instr_posns, decl_blocks,
            rpo_index, cur_block_id,
            num_starts: 0_u16, num_loops: 0_u16,
            total_phis: 0_u32, num_phis: 0_u32
        };

        // Declare a start block and enter it
        // immediately.
        let start_id = bs.decl_start_block();
        unsafe {
            bs.enter_block(start_id);
        };
        bs
    }

    pub(crate) fn start_block_id(&self) -> BlockId {
        debug_assert!(self.decl_blocks.len() > 0);
        BlockId(0)
    }

    pub(crate) fn instr_bytes_len(&self) -> usize {
        self.instr_bytes.len()
    }

    fn front_instr_posn(&self) -> InstrPosn {
        InstrPosn::new(self.instr_bytes.len() as u32)
    }
    fn front_instr_no(&self) -> InstrNo {
        InstrNo::new(self.instr_posns.len() as u32)
    }

    // Declare a new block and get an index for it.
    fn decl_block(&mut self, bv: BlockVariant) -> BlockId {
        let len = self.decl_blocks.len() as u32;
        if len >= Self::MAX_DECL_BLOCKS {
            panic!("Too many declared blocks.");
        }
        let id = BlockId(len);
        self.decl_blocks.push(Block::new(id, bv));
        id
    }

    pub(crate) fn total_blocks(&self) -> usize {
        self.decl_blocks.len()
    }
    pub(crate) fn iter_blocks(&self)
      -> impl Iterator<Item=&Block>
    {
        self.decl_blocks.iter()
    }

    pub(crate) fn decl_plain_block(&mut self,
        num_phis: u32)
      -> BlockId
    {
        let id = self.decl_block(
          BlockVariant::Plain { num_phis });

        // Update `total_phis` to reflect the phis in
        // this block.
        self.total_phis += num_phis;

        id
    }
    pub(crate) fn decl_start_block(&mut self) -> BlockId {
        // Assign a new start block number.
        let start_no = self.num_starts;
        let id = self.decl_block(
          BlockVariant::Start { start_no });
        self.num_starts += 1;
        id
    }
    pub(crate) fn decl_loop_head(&mut self, num_phis: u32)
      -> BlockId
    {
        // Assign a new loop number.
        let loop_no = self.num_loops;

        // Restrict loop_no from being 0xffff, because
        // that's the sentinel "uninitialized" value.
        assert!(loop_no < u16::max_value());

        let id = self.decl_block(
          BlockVariant::Loop { num_phis, loop_no });
        self.num_loops += 1;

        // Update total phis to include this block's phis.
        self.total_phis += num_phis;

        id
    }

    pub(crate) unsafe fn get_block(&self, id: BlockId)
      -> &Block
    {
        debug_assert!((id.0 as usize)
                        < self.decl_blocks.len());
        self.decl_blocks.get_unchecked(id.0 as usize)
    }
    pub(crate) unsafe fn get_mut_block(&mut self,
        id: BlockId)
      -> &mut Block
    {
        debug_assert!((id.0 as usize)
                        < self.decl_blocks.len());
        self.decl_blocks.get_unchecked_mut(id.0 as usize)
    }

    // Start specifying a block.  Unsafe for unchecked
    // access to the declared blocks vec.
    pub unsafe fn enter_block(&mut self, id: BlockId) {
        // First instr for block is the front instr.
        let first_ins_pos = self.front_instr_posn();
        let first_ins = self.front_instr_no();

        // Compute global ordering of block.
        let order: u32 = self.rpo_index.len() as u32;

        // Mark new block as entered.
        self.get_mut_block(id)
            .set_entered(order, first_ins);

        // Add the id of the block to the RPO vec.
        debug_assert!(! self.rpo_index.contains(&id));
        self.rpo_index.push(id);

        // Set the current block.
        self.cur_block_id = id;

        debug!("Enter block id={} first_ins={}/{}, ord={}",
               id.as_u32(), first_ins, first_ins_pos,
               order);
    }

    // Finish specifying a block.
    unsafe fn finish_block(&mut self,
        id: BlockId, last_ins: InstrNo)
    {
        // Update the block state.
        self.get_mut_block(id).set_finished(last_ins);
    }

    // Finish specifying a block.
    pub(crate) unsafe fn finish_loop(&mut self,
        id: BlockId)
    {
        // Update the block state.
        self.get_mut_block(id).set_loop_complete();
    }

    pub(crate) fn take_instr_no(&mut self, posn: InstrPosn)
      -> InstrNo
    {
        let instr_no = self.instr_posns.len();
        debug_assert!(
          instr_no < (Self::MAX_INSTRS as usize));
        self.instr_posns.push(posn);
        InstrNo::new(instr_no as u32)
    }
    pub(crate) fn take_phi_no(&mut self) -> u32 {
        let phi_no = self.num_phis;
        self.num_phis += 1;
        phi_no
    }

    pub(crate) fn emit_instr<I: Instr>(&mut self,
        instr: I)
      -> Option<(BlockId, InstrNo)>
    {
        // Get the location and number of the instr.
        let block_id = self.cur_block_id;
        let instr_posn = self.front_instr_posn();
        let instr_no = self.take_instr_no(instr_posn);

        // Encode the instruction bytes.
        let sz = instr.send_instr(&mut self.instr_bytes) ?;

        unsafe { self.get_mut_block(block_id) }
          .set_add_instr(instr_no);

        debug_assert!(
          (instr_posn.as_u32() + (sz as u32))
            == (self.instr_bytes.len() as u32));

        debug!("Instr {} {}", instr_posn, instr_no);

        // Return the block and instruction.
        Some((block_id, instr_no))
    }

    pub(crate) fn emit_end<EI: EndInstr>(&mut self,
        end_instr: EI)
      -> Option<(BlockId, InstrNo)>
    {
        // Get the location and number of the instr.
        let block_id = self.cur_block_id;
        let instr_posn = self.front_instr_posn();
        let instr_no = self.take_instr_no(instr_posn);

        // Encode the instruction data.
        let sz =
          end_instr.send_end(&mut self.instr_bytes) ?;

        // increment target blocks input edge count.
        for tgt_pair in end_instr.targets() {
            let tgt_id = tgt_pair.0.into();
            let num_phi_defns = tgt_pair.1.len();
            let tgt_ref = unsafe {
              self.get_mut_block(tgt_id)
            };

            // Verify that the targets are valid.

            // If the target block has been entered
            // already, it must be a loop head block
            // for an incomplete loop.
            if tgt_ref.has_entered() {
                if ! tgt_ref.is_loop() {
                    panic!("Backjump to nonloop block {}",
                           tgt_id.as_u32());
                }
                if tgt_ref.has_loop_complete() {
                    panic!("Backjump to complete loop \
                            with head block {}",
                           tgt_id.as_u32());
                }
            }

            // Increment the target's input edge
            // count.
            tgt_ref.incr_input_edges();
            debug!("  => target {} phis={}",
                   tgt_id.as_u32(), num_phi_defns);
        }

        // Finish the current block.
        unsafe { self.finish_block(block_id, instr_no) };

        debug_assert!(
          (instr_posn.as_u32() + (sz as u32))
            == (self.instr_bytes.len() as u32));

        debug!("End Instr {} {}", instr_posn, instr_no);

        debug!("End Block {} @{}",
               block_id.as_u32(), self.instr_bytes.len());

        Some((block_id, instr_no))
    }
}
