
use std::fmt;

use crate::ops::{ Opcode, Operation, Op };
use crate::ir_types::IrTypeId;

/**
 * The branch instruction branches on a boolean
 * operand, selecting one of two target blocks.
 */
#[derive(Clone)]
pub struct JumpOp;

impl JumpOp {
    pub(crate) fn new() -> JumpOp { JumpOp }
}

impl Operation for JumpOp {
    fn opcode() -> Opcode { Opcode::Jump }
    fn terminal() -> bool { true }
    fn op(&self) -> Op { Op::Jump(self.clone()) }
    fn out_type(&self) -> Option<IrTypeId> { None }
    fn num_operands(&self) -> u32 { 0 }
    fn num_targets(&self) -> Option<u32> { Some(1) }

    fn write_to(&self, vec: &mut Vec<u8>) {}

    unsafe fn read_from(_bytes: &[u8]) -> (usize, Self) {
        (0, JumpOp::new())
    }
}

impl fmt::Display for JumpOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Jump")
    }
}
