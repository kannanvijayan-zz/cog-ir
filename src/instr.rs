
use std::fmt;
use std::str::from_utf8;
use std::fmt::Write;

use crate::ops::{ Operation, Op };
use crate::block::BlockId;
use crate::defn::Defn;

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
 * Stores information about a decoded instruction.
 */
pub(crate) struct InstrInfo<'a> {
    /** The instruction data. */
    instr_data: &'a [u8],

    /** The id of the instruction. */
    defn: Defn<'a>,

    /** The operation. */
    op: Op,

    /** The offset to the input operands list. */
    inputs_offset: u32,

    /** The offset to after the operands list.
     * Also the offset to the targets list for
     * end instructions. */
    after_inputs_offset: u32,
}

/**
 * An InstrInputs iterates through the input
 * definitions for an instruction.
 */
pub struct InstrInputs<'a> {
    // Remaining # of inputs to read.
    remaining: u32,

    // Number of bytes read so far.
    bytes_read: u32,

    // The current bytes cursor.
    bytes: &'a [u8]
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

    unsafe fn instr_data(&self, id: InstrId) -> &[u8] {
        let offset = id.posn().as_u32();
        debug_assert!((offset as usize )
                        <= self.instr_bytes.len());
        &self.instr_bytes[offset as usize ..]
    }
    pub(crate) unsafe fn read_instr_info<'a>(
        &'a self, instr_id: InstrId)
      -> InstrInfo<'a>
    {
        let instr_data = self.instr_data(instr_id);
        let defn = Defn::new(instr_id);
        let (nb, op) = Op::read_from(instr_data);
        let inputs_offset = nb as u32;
        let after_inputs_offset = 0;
        let mut instr_info = InstrInfo {
            instr_data, defn, op,
            inputs_offset, after_inputs_offset,
        };

        // Adjust after_inputs_offset to be correct.
        let mut inputs_iter = instr_info.inputs_iter();
        loop {
            if inputs_iter.next() == None { break; }
        }
        instr_info.after_inputs_offset =
            inputs_offset + inputs_iter.bytes_read();
        instr_info
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

impl<'a> InstrInfo<'a> {
    pub(crate) fn defn(&self) -> Defn<'a> { self.defn }

    fn inputs_data(&self) -> &'a [u8] {
        let offset = self.inputs_offset as usize;
        debug_assert!(self.instr_data.len() >= offset);
        unsafe { self.instr_data.get_unchecked(offset..) }
    }

    pub(crate) fn op(&self) -> &Op { &self.op }
    pub(crate) fn inputs_iter(&self) -> InstrInputs<'a> {
        unsafe {
            InstrInputs::new(
              self.op.num_inputs(), self.inputs_data())
        }
    }

    pub(crate) fn next_defn(&self) -> Option<Defn<'a>> {
        if self.op().terminal() {
            return None;
        }
        // Use the after_inputs_offset.
        let instr_offset = self.defn.instr_id().as_u32();
        let next_instr_offset =
          instr_offset + self.after_inputs_offset;
        debug_assert!((self.after_inputs_offset as usize)
                        < self.instr_data.len());

        let posn = InstrPosn::new(next_instr_offset);
        Some(Defn::new(InstrId::new(posn)))
    }
}
impl<'a> fmt::Display for InstrInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "InstrInfo[{} - {}]",
          self.defn(), self.op())
    }
}

impl<'a> InstrInputs<'a> {
    // Unsafe constructing this because the safe
    // iterator implementation uses unsafe code.
    unsafe fn new(remaining: u32, bytes: &'a [u8])
      -> InstrInputs<'a>
    {
        InstrInputs { remaining, bytes, bytes_read: 0 }
    }

    fn bytes_read(&self) -> u32 { self.bytes_read }
}
impl<'a> Iterator for InstrInputs<'a> {
    type Item = Defn<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        let (nb, id64) = unsafe {
            leb128::read_leb128u(self.bytes)
        };
        self.bytes_read += nb as u32;
        self.remaining -= 1;
        self.bytes = unsafe {
          self.bytes.get_unchecked(nb ..)
        };
        let posn = InstrPosn::new(id64 as u32);
        Some(Defn::new(InstrId::new(posn)))
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
    fn posn(&self) -> InstrPosn { self.0 }
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

