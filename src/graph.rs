
use std::mem;

use crate::block::{ Block, BlockId, BlockRef, BlockStore };
use crate::ops::{ Op };
use crate::instr::{
    InstrId, InstrStore, InstrInfo, InstrInputs
};
use crate::defn::{ Defn, TypedDefn };
use crate::leb128;

/**
 * A Graph represents a fully constructed graph.
 */
pub struct Graph {
    // The instruction store.
    instr_store: InstrStore,

    // The block store.
    block_store: BlockStore,
}

impl Graph {
    pub(crate) fn new(
        instr_store: InstrStore, block_store: BlockStore)
      -> Graph 
    {
        Graph { instr_store, block_store }
    }

    pub fn dump_stats(&self, name: &'static str) {
        debug!("Graph {} instrs={} blocks={}",
               name,
               self.instr_store.instr_bytes_len(),
               self.block_store.total_blocks());
    }

    pub fn enter_session<R, F>(&self, f: F) -> R
      where F: for <'gs> FnOnce (&mut GraphSession<'gs>)
                            -> R
    {
        // Enter the start block automatically.
        let start_block_id =
          self.block_store.start_block_id();

        let mut sess = GraphSession::new(
          self, BlockRef::new(start_block_id));

        f(&mut sess)
    }
}

/**
 * A graph reader is parameterized around the lifetime
 * of a graph, and allows for safe access into the
 * elements of the graph (blocks, instructions).
 */
pub struct GraphSession<'gs> {
    // The underlying graph.
    graph: &'gs Graph,

    // Current block being read.
    cur_block: BlockRef<'gs>,

    // Current instruction being read.
    cur_instr: InstrInfo<'gs>
}

impl<'gs> GraphSession<'gs> {
    fn new(graph: &'gs Graph,
           cur_block: BlockRef<'gs>)
      -> GraphSession<'gs>
    {
        let cur_instr = unsafe {
            let blk = 
              graph.block_store.get_block(cur_block.id());
            debug_assert!(blk.has_finished());

            graph.instr_store.read_instr_info(
              blk.first_instr())
        };

        GraphSession { graph, cur_block, cur_instr }
    }

    // Retrieve the a reference to the actual
    // block from a `BlockRef` index.
    fn get_block(&self, block: BlockRef<'gs>) -> &Block {
        unsafe {
            self.graph.block_store.get_block(block.id())
        }
    }
    fn get_cur_block(&self) -> &Block {
        self.get_block(self.cur_block)
    }

    // Get the current op.
    pub(crate) fn cur_op(&self) -> &Op {
        self.cur_instr.op()
    }

    // Read the current instruction input definitions
    // into the given slice.
    pub(crate) fn cur_inputs(&self) -> InstrInputs<'gs> {
        self.cur_instr.inputs_iter()
    }

    // Go to the next instruction, returning its Defn.
    // If at the last instruction, None is returned.
    pub fn next_defn(&mut self) -> Option<Defn<'gs>> {
        let nxdef = self.cur_instr.next_defn() ?;
        // Get the instr_info for the next definition.
        self.cur_instr = unsafe {
          self.graph.instr_store.read_instr_info(
            nxdef.instr_id())
        };

        Some(nxdef)
    }

    pub fn debug_print_cur_instr(&self) {
        let bl = self.get_cur_block();
        debug!("{} {} - {}",
            bl.id(), self.cur_instr.defn(), self.cur_op());
    }
}
