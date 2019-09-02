
use std::fmt;

use crate::ops::Opcode;
use crate::ir_types::IrOutputType;

/**
 * An operation embodies the full notion of an
 * instruction's actions, without identifying any
 * of the specific associated input definitions.
 */
pub trait Operation: Sized + Clone + fmt::Display {
    /** The type of the output for the operation. */
    type Output: IrOutputType;

    /** Get the opcode for this operation. */
    fn opcode() -> Opcode;

    /** Get the number of expected operands. */
    fn num_operands(&self) -> u32;

    /** Write to a vec. */
    fn write_to(&self, vec: &mut Vec<u8>);

    /** Read from some bytes, unchecked. */
    unsafe fn read_from(bytes: &[u8]) -> (usize, Self);
}

/** An terminal operation terminates a block. */
pub trait TerminalOperation: Operation {
    /** The number of target blocks for this operation. */
    fn num_targets(&self) -> u32;
}
