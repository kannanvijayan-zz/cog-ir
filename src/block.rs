
use std::fmt;
use std::marker::PhantomData;

use crate::instr::{ InstrId, InstrPosn };

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
impl fmt::Display for BlockId {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "BlockId({})", self.0)
    }
}

/** A reference to a block. */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
pub struct BlockRef<'a>(BlockId, PhantomData<&'a ()>);

impl<'a> BlockRef<'a> {
    pub(crate) fn new(id: BlockId) -> BlockRef<'a> {
        BlockRef(id, Default::default())
    }

    pub(crate) fn id(&self) -> BlockId { self.0 }
}

impl<'a> Into<BlockId> for BlockRef<'a> {
    fn into(self) -> BlockId { self.0 }
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

    // The numbering of the block in specification order
    // (RPO).  Only set when the block is entered.
    order: u32,

    // The index of the first instruction.  Only set
    // when the block is entered.
    first_instr: InstrId,

    // The index of the last (end) instruction.  Only set
    // when end instruction is emitted and the block is
    // finished.
    last_instr: InstrId,
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
          first_instr: InstrId::invalid(),
          last_instr: InstrId::invalid()
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

    pub(crate) fn num_phis(&self) -> u32 {
        match self.variant {
          BlockVariant::Plain { num_phis }
            => num_phis,
          BlockVariant::Loop { num_phis, loop_no: _ }
            => num_phis,
          BlockVariant::Start { start_no }
            => 0
        }
    }

    pub(crate) fn first_instr(&self) -> InstrId {
        self.first_instr
    }
    pub(crate) fn last_instr(&self) -> InstrId {
        self.last_instr
    }

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
      order: u32, first_instr: InstrId)
    {
        debug_assert!(! self.has_entered());
        self.state = BlockState::Entered;
        self.first_instr = first_instr;
        self.order = order;
    }
    fn set_add_instr(&mut self, last_instr: InstrId) {
        debug_assert!(self.has_entered());
        debug_assert!(! self.has_finished());
        self.last_instr = last_instr;
    }
    fn set_finished(&mut self, last_instr: InstrId) {
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
    decl_blocks: Vec<Block>,
    rpo_index: Vec<BlockId>,
    cur_block_id: BlockId,
    num_starts: u16,
    num_loops: u16,
    total_phis: u32,
}

impl BlockStore {
    const INSTR_BYTES_CAP: usize = 256;
    const INSTR_POSNS_CAP: usize = 256;
    const DECL_BLOCKS_CAP: usize = 8;
    const RPO_INDEX_CAP: usize = 8;

    const MAX_DECL_BLOCKS: u32 = 0xf_ffff;
    const MAX_INSTR_BYTES: u32 = 0xff_ffff;

    pub fn new() -> BlockStore {
        let decl_blocks =
          Vec::with_capacity(Self::DECL_BLOCKS_CAP);
        let rpo_index =
          Vec::with_capacity(Self::RPO_INDEX_CAP);

        let cur_block_id = BlockId(0);

        let mut bs = BlockStore {
            decl_blocks, rpo_index, cur_block_id,
            num_starts: 0_u16, num_loops: 0_u16,
            total_phis: 0_u32
        };

        // Declare a start block and enter it
        // immediately.
        let first_id = bs.decl_start_block();
        let first_ins = InstrId::new(InstrPosn::new(0));
        unsafe { bs.enter_block(first_id, first_ins) };

        bs
    }

    pub(crate) fn start_block_id(&self) -> BlockId {
        debug_assert!(self.decl_blocks.len() > 0);
        BlockId(0)
    }
    pub(crate) unsafe fn last_rpo_block(&self)
      -> BlockId
    {
        debug_assert!(self.rpo_index.len()
                        == self.decl_blocks.len());
        debug_assert!(self.rpo_index.len() > 0);
        let last_block_id =
            *self.rpo_index.get_unchecked(
              self.rpo_index.len() - 1);
        debug_assert!(
          self.get_block(last_block_id).has_finished());
        last_block_id
    }
    pub(crate) unsafe fn next_rpo_block(
        &self, block_id: BlockId)
      -> Option<BlockId>
    {
        let block = self.get_block(block_id);
        debug_assert!(block.has_finished());
        let ord = block.order as usize;
        debug_assert!(ord < self.rpo_index.len());
        let next_block_id =
          *(self.rpo_index.get(ord + 1) ?);
        debug_assert!(
          self.get_block(next_block_id).has_finished());
        Some(next_block_id)
    }
    pub(crate) unsafe fn prior_rpo_block(
        &self, block_id: BlockId)
      -> Option<BlockId>
    {
        let block = self.get_block(block_id);
        debug_assert!(block.has_finished());
        let ord = block.order as usize;
        debug_assert!(ord < self.rpo_index.len());
        if ord > 0 {
            let prior_block_id =
              *self.rpo_index.get_unchecked(ord - 1);
            debug_assert!(
              self.get_block(prior_block_id)
                .has_finished());
            Some(prior_block_id)
        } else {
            None
        }
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

    pub(crate) fn decl_plain_block(
        &mut self, num_phis: u32)
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
    pub(crate) unsafe fn get_mut_block(
        &mut self, id: BlockId)
      -> &mut Block
    {
        debug_assert!((id.0 as usize)
                        < self.decl_blocks.len());
        self.decl_blocks.get_unchecked_mut(id.0 as usize)
    }

    // Start specifying a block.  Unsafe for unchecked
    // access to the declared blocks vec.
    pub(crate) unsafe fn enter_block(
        &mut self, id: BlockId, first_ins: InstrId)
    {
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

        debug!("Enter block id={} first_ins={} ord={}",
               id.as_u32(), first_ins, order);
    }

    // Finish specifying a block.
    pub(crate) unsafe fn finish_block(
        &mut self, id: BlockId, last_ins: InstrId)
    {
        // Update the block state.
        self.get_mut_block(id).set_finished(last_ins);
    }

    // Finish specifying a block.
    pub(crate) unsafe fn finish_loop(
        &mut self, id: BlockId)
    {
        // Update the block state.
        self.get_mut_block(id).set_loop_complete();
    }
}
