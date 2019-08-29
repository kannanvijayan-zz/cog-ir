
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
pub struct JumpOp;

impl JumpOp {
    pub(crate) fn new() -> JumpOp { JumpOp }
}

impl TerminalOperation for JumpOp {
    fn num_targets(&self) -> usize { 1 }
}
impl Operation for JumpOp {
    type Output = VoidTy;

    fn opcode(&self) -> Opcode { Opcode::Jump }

    fn num_operands(&self) -> u32 { 0 }
    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        sink.send_slice("Jump")
    }
}

impl ByteSerialize for JumpOp {
    fn send_to<S>(&self, _sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        Some(0)
    }

    unsafe fn take_from<S>(_src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
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
