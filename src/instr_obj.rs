
use std::fmt;

use crate::ops::{ Operation, TerminalOperation };
use crate::block::BlockId;
use crate::instr::{ InstrId, Instr, EndInstr };

/** Instruction objects are lifetime-restricted
 * wrappers around an operation and a list of
 * definitions.
 *
 * This type specializes the `Instr` trait, and is mostly
 * used via generalization via that.
 */
pub struct InstrObj<'a, OP, DEF>(&'a OP, &'a [DEF])
  where OP: Operation,
        DEF: Copy + Into<InstrId>;

/** End instruction objects are lifetime-restricted
 * wrappers around a terminal operation and a list of
 * definitions, as well as a list of target blocks
 * with their phi-argument definition list.
 *
 * This type specializes the `EndInstr` trait, and is
 * mostly used via generalization via that.
 */
pub struct EndInstrObj<'a, OP, DEF, BLK>(
    &'a OP, &'a [DEF], &'a [(BLK, &'a [DEF])])
  where OP: TerminalOperation + Operation,
        DEF: Copy + Into<InstrId>,
        BLK: Copy + Into<BlockId>;


// Main interface for InstrObj
impl<'a, OP, DEF> InstrObj<'a, OP, DEF>
  where OP: Operation,
        DEF: Copy + Into<InstrId>
{
    pub(crate) fn new(op: &'a OP, inputs: &'a [DEF])
      -> InstrObj<'a, OP, DEF>
    {
        assert!(
          (op.num_operands() as usize) == inputs.len());
        InstrObj(op, inputs)
    }
}

// Main interface for EndInstrObj
impl<'a, OP, DEF, BLK> EndInstrObj<'a, OP, DEF, BLK>
  where OP: TerminalOperation + Operation,
        DEF: Copy + Into<InstrId>,
        BLK: Copy + Into<BlockId>
{
    pub(crate) fn new(op: &'a OP, inputs: &'a [DEF],
                      targets: &'a [(BLK, &'a [DEF])])
      -> EndInstrObj<'a, OP, DEF, BLK>
    {
        assert!(
          (op.num_operands() as usize) == inputs.len());
        assert!(
          (op.num_targets() as usize) == targets.len());
        EndInstrObj(op, inputs, targets)
    }
}

// Implement Instr for InstrObj
impl<'a, OP, DEF> Instr for InstrObj<'a, OP, DEF>
  where OP: Operation,
        DEF: Copy + Into<InstrId>
{
    type Op = OP;
    type Def = DEF;

    fn op(&self) -> &Self::Op { &self.0 }
    fn inputs(&self) -> &[Self::Def] { &self.1 }
}

// Implement EndInstr for EndInstrObj
impl<'a, OP, DEF, BLK> EndInstr
  for EndInstrObj<'a, OP, DEF, BLK>
  where OP: TerminalOperation + Operation,
        DEF: Copy + Into<InstrId>,
        BLK: Copy + Into<BlockId>
{
    type Op = OP;
    type Def = DEF;
    type Blk = BLK;

    fn op(&self) -> &Self::Op { &self.0 }
    fn inputs(&self) -> &[Self::Def] { &self.1 }
    fn targets(&self) -> &[(Self::Blk, &[Self::Def])] {
        &self.2
    }
}
