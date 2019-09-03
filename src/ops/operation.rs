
use std::fmt;

use crate::ops::Opcode;
use crate::ops::Op;
use crate::ir_types::IrTypeId;

/**
 * An operation embodies the full notion of an
 * instruction's actions, without identifying any
 * of the specific associated input definitions.
 */
pub trait Operation: Sized + Clone + fmt::Display {
    /** Get the opcode for this operation. */
    fn opcode() -> Opcode;

    /** Check if the operation is terminal. */
    fn terminal() -> bool { false }

    /** Get the op for this operation. */
    fn op(&self) -> Op;

    /** Get the output type of this operation. */
    fn out_type(&self) -> Option<IrTypeId>;

    /** Get the number of expected operands. */
    fn num_operands(&self) -> u32;

    /** The number of target blocks for this operation,
        only valid for a terminal operation. */
    fn num_targets(&self) -> Option<u32> { None }

    /** Write to a vec. */
    fn write_to(&self, vec: &mut Vec<u8>);

    /** Read from some bytes, unchecked. */
    unsafe fn read_from(bytes: &[u8]) -> (usize, Self);
}
