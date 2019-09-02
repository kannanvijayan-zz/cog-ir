
use std::fmt;

use crate::ops::{ Opcode, Operation };
use crate::ir_types::VoidTy;

/**
 * The Nop instr does nothing.
 */
#[derive(Clone)]
pub struct NopOp;

impl NopOp {
    pub(crate) fn new() -> NopOp { NopOp }
}

impl Operation for NopOp {
    type Output = VoidTy;

    fn opcode() -> Opcode { Opcode::Nop }

    fn num_operands(&self) -> u32 { 0 }

    fn write_to(&self, vec: &mut Vec<u8>) {}

    unsafe fn read_from(_bytes: &[u8]) -> (usize, Self) {
        (0, NopOp::new())
    }
}

impl fmt::Display for NopOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Nop")
    }
}
