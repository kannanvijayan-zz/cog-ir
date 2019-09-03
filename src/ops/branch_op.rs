
use std::fmt;

use crate::ops::{ Opcode, Operation, TerminalOperation };
use crate::ir_types::IrTypeId;

/**
 * The branch instruction branches on a boolean
 * operand, selecting one of two target blocks.
 */
#[derive(Clone)]
pub struct BranchOp;

impl BranchOp {
    pub(crate) fn new() -> BranchOp { BranchOp }
}

impl TerminalOperation for BranchOp {
    fn num_targets(&self) -> u32 { 2 }
}
impl Operation for BranchOp {
    fn opcode() -> Opcode { Opcode::Branch }
    fn out_type(&self) -> Option<IrTypeId> { None }
    fn num_operands(&self) -> u32 { 1 }

    fn write_to(&self, vec: &mut Vec<u8>) {}

    unsafe fn read_from(_bytes: &[u8]) -> (usize, Self) {
        (0, BranchOp::new())
    }
}

impl fmt::Display for BranchOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Branch")
    }
}
