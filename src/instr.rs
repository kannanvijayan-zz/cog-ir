
use std::fmt;
use std::str::from_utf8;
use std::fmt::Write;

use crate::ops::{ Operation, Op };
use crate::block::BlockId;

use crate::leb128;

/** Stores a writable instruction stream and presents
 * an API to write (append-only) instructions to it,
 * and to read from it. */
pub(crate) struct InstrStore {
    /** The raw instruction bytes. */
    instr_bytes: Vec<u8>,

    /** Max len of vec. */
    max_len: u32,

    /** The number of instructions emitted. */
    num_instrs: u32,
}

/**
 * The offset of an instruction in the instruction stream.
 * Serves as the canonical id for an instruction.
 */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrPosn(u32);

/**
 * The id of an instruction is just its position.
 */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrId(InstrPosn);

impl InstrStore {
    const INIT_INSTR_BYTES: usize = 256;
    const MAX_INSTR_BYTES: usize = 0xff_ffff;

    pub(crate) fn new() -> InstrStore {
        let max_len = Self::MAX_INSTR_BYTES as u32;
        let instr_bytes =
          Vec::with_capacity(Self::INIT_INSTR_BYTES);
        InstrStore { instr_bytes, max_len, num_instrs: 0 }
    }

    fn within_limits(&self) -> bool {
        self.instr_bytes.len() <= (self.max_len as usize)
    }

    fn front_instr_posn(&self) -> InstrPosn {
        debug_assert!(self.within_limits());
        InstrPosn::new(self.instr_bytes.len() as u32)
    }
    pub(crate) fn front_instr_id(&self) -> InstrId {
        InstrId::new(self.front_instr_posn())
    }

    pub(crate) fn instr_bytes_len(&self) -> usize {
        self.instr_bytes.len()
    }

    fn append_instr_impl<OP, DEF>(
        &mut self, op: &OP, inputs: &[DEF])
      where OP: Operation, DEF: Copy + Into<InstrId>
    {
        debug_assert!(self.within_limits());

        // Encode the opcode for the instruction.
        self.instr_bytes.push(
          OP::opcode().into_u8());

        // Encode the operation payload.
        op.write_to(&mut self.instr_bytes);

        // Encode each operand.
        for inp in inputs {
            leb128::write_leb128u(
                (*inp).into().as_u32(),
                &mut self.instr_bytes);
        }
    }

    fn append_targets_impl<BLK, DEF>(
        &mut self, targets: &[(BLK, &[DEF])])
      where BLK: Copy + Into<BlockId>,
            DEF: Copy + Into<InstrId>
    {
        for &(target_blk, phi_defs) in targets.iter() {
            Self::debug_print_target(target_blk, phi_defs);

            // Write the target block-id.
            leb128::write_leb128u(
              target_blk.into().as_u32(),
              &mut self.instr_bytes);

            // Write out # of phi-defs.
            debug_assert!(
              phi_defs.len() <= Self::MAX_INSTR_BYTES);
            leb128::write_leb128u(
                phi_defs.len() as u32,
                &mut self.instr_bytes);

            // Write out each phi def for the target.
            for def in phi_defs {
                leb128::write_leb128u(
                    (*def).into().as_u32(),
                    &mut self.instr_bytes);
            }
        }
    }

    fn debug_print_instr<OP, DEF>(
        id: InstrId, op: &OP, inputs: &[DEF])
      where OP: Operation,
            DEF: Copy + Into<InstrId>
    {
        let mut inputs_str = String::new();
        for (i, def) in inputs.iter().enumerate() {
            if i > 0 {
                write!(inputs_str, ", ").unwrap();
            }
            write!(inputs_str, "{}",
                   (*def).into().as_u32()).unwrap();
        }
        if inputs_str.len() > 0 {
            debug!("Emit {} - {}({})",
                   id.as_u32(), op, inputs_str);
        } else {
            debug!("Emit {} - {}", id.as_u32(), op);
        }
    }

    fn debug_print_target<BLK, DEF>(
        target: BLK, phi_args: &[DEF])
      where BLK: Into<BlockId>,
            DEF: Copy + Into<InstrId>
    {
        let mut phi_args_str = String::new();
        for (i, def) in phi_args.iter().enumerate() {
            if i > 0 {
                write!(phi_args_str, ", ").unwrap();
            }
            write!(phi_args_str, "{}:{}",
                   i, (*def).into().as_u32()).unwrap();
        }
        if phi_args_str.len() > 0 {
            debug!("  Target {} - {}",
                   target.into().as_u32(), phi_args_str);
        } else {
            debug!("  Target {}", target.into().as_u32());
        }
    }

    pub(crate) fn emit_instr<OP, DEF>(
        &mut self, op: &OP, inputs: &[DEF])
      -> Option<InstrId>
      where OP: Operation,
            DEF: Copy + Into<InstrId>
    {
        debug_assert!(! OP::terminal());

        if ! self.within_limits() { return None; }

        // Save the offset of the instruction
        let id = self.front_instr_id();

        Self::debug_print_instr(id, op, inputs);

        // Append the instruction encoding, and
        // the list of input operands.
        self.append_instr_impl(op, inputs);

        if ! self.within_limits() { return None; }

        Some(id)
    }

    pub(crate) fn emit_end<OP, DEF, BLK>(
        &mut self,
        op: &OP,
        inputs: &[DEF],
        targets: &[(BLK, &[DEF])])
      -> Option<InstrId>
      where OP: Operation,
            DEF: Copy + Into<InstrId>,
            BLK: Copy + Into<BlockId>
    {
        debug_assert!(OP::terminal());

        if ! self.within_limits() { return None; }

        // Save the offset of the instruction
        let id = self.front_instr_id();

        Self::debug_print_instr(id, op, inputs);

        // Append the instruction encoding, and
        // the list of input operands.
        self.append_instr_impl(op, inputs);

        // Append the (target, phi_defs) list.
        self.append_targets_impl(targets);

        if ! self.within_limits() { return None; }

        Some(id)
    }
}

impl InstrPosn {
    const INVALID_VALUE: u32 = u32::max_value();

    pub(crate) fn new(val: u32) -> InstrPosn {
        debug_assert!(val != Self::INVALID_VALUE);
        InstrPosn(val)
    }
    pub(crate) fn invalid() -> InstrPosn {
        InstrPosn(Self::INVALID_VALUE)
    }
    pub(crate) fn as_u32(&self) -> u32 {
        debug_assert!(self.0 != Self::INVALID_VALUE);
        self.0
    }
}

impl InstrId {
    const INVALID_VALUE: u32 = u32::max_value();

    pub(crate) fn new(posn: InstrPosn) -> InstrId {
        InstrId(posn)
    }
    pub(crate) fn as_u32(&self) -> u32 { self.0.as_u32() }

    pub(crate) fn invalid() -> InstrId {
        InstrId(InstrPosn::invalid())
    }
}
impl fmt::Display for InstrId {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "[Ins@{}]", self.0.as_u32())
    }
}

