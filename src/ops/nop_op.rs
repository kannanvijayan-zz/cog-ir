
use std::fmt;

use crate::byte_sink::{
    ByteSink, ByteSource, ByteSerialize
};
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
}

impl ByteSerialize for NopOp {
    fn send_to<S>(&self, _sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        Some(0)
    }

    unsafe fn take_from<S>(_src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
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
