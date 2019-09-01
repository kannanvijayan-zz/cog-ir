
use std::fmt;

use crate::ops::Opcode;
use crate::ir_types::IrOutputType;
use crate::byte_sink::{ ByteSink, ByteSerialize };

/**
 * An operation embodies the full notion of an
 * instruction's actions, without identifying any
 * of the specific associated input definitions.
 */
pub trait Operation: Sized + Clone + fmt::Display
                     + ByteSerialize
{
    /** The type of the output for the operation. */
    type Output: IrOutputType;

    /** Get the opcode for this operation. */
    fn opcode() -> Opcode;

    /** Get the number of expected operands. */
    fn num_operands(&self) -> u32;

    /** Write out the operation's description. */
    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink;
}

/** An terminal operation terminates a block. */
pub trait TerminalOperation: Operation {
    /** The number of target blocks for this operation. */
    fn num_targets(&self) -> u32;
}
