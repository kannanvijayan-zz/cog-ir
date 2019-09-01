
use std::fmt;

use crate::byte_sink::{
    ByteSink, ByteSource, ByteSerialize
};
use crate::ops::{ Opcode, Operation, TerminalOperation };
use crate::ir_types::VoidTy;

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
    type Output = VoidTy;

    fn opcode() -> Opcode { Opcode::Branch }

    fn num_operands(&self) -> u32 { 1 }
    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        sink.send_slice("Branch")
    }
}

impl ByteSerialize for BranchOp {
    fn send_to<S>(&self, _sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        Some(0)
    }

    unsafe fn take_from<S>(_src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
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
