
use std::fmt;

use crate::ops::{ Operation, TerminalOperation };
use crate::block::BlockId;
use crate::byte_sink::{ ByteSink, ByteSerialize, Leb128U };
use crate::instr_obj::InstrObj;

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

/** Instr is a helper trait handle `InstrObj` values,
 * but internalize the operation and definition types,
 * for code hygiene purposes.
 *
 * The only specializer is `InstrObj`, which exposes
 * the operation and definition types in the type-name.
 */
pub trait Instr {
    type Op: Operation;
    type Def: Copy + Into<InstrId>;

    fn op(&self) -> &Self::Op;
    fn inputs(&self) -> &[Self::Def];

    fn send_instr<S: ByteSink>(&self, sink: &mut S)
      -> Option<usize>
    {
        // Encode the opcode for the instruction.
        let mut sz = sink.send_byte(
          Self::Op::opcode().into_u8()) ?;

        // Encode the operation payload.
        sz += self.op().send_to(sink) ?;

        // Encode each operand.
        for opnd in self.inputs() {
            let opnd_id = (*opnd).into();
            sz +=
              Leb128U::from(opnd_id.as_u32())
                .send_to(sink) ?;
        }

        Some(sz)
    }
}

/**
 * Similar to the `Instr` trait, this trait
 * specialized for `EndInstrObj` values, and
 * internalizes the `Op`, `Def`, and `Blk` traits,
 * for ease of reference in code.
 */
pub trait EndInstr {
    type Op: TerminalOperation + Operation;
    type Def: Copy + Into<InstrId>;
    type Blk: Copy + Into<BlockId>;

    fn op(&self) -> &Self::Op;
    fn inputs(&self) -> &[Self::Def];
    fn targets(&self) -> &[(Self::Blk, &[Self::Def])];

    fn send_end<S: ByteSink>(&self, sink: &mut S)
      -> Option<usize>
    {
        // Send the main instruction body first.
        let mut sz =
          InstrObj::new(self.op(), self.inputs())
            .send_instr(sink) ?;

        // Encode target blocks and their input phis.
        for tgt_pair in self.targets() {
            let tgt_id = tgt_pair.0.into();
            let phi_defns = tgt_pair.1;

            // Write out the target block id.
            sz += Leb128U::from(tgt_id.as_u32())
                          .send_to(sink) ?;

            // Write out all the phi definitions for this
            // block, prefixed with their count.
            // TODO: ensure that defn_list is small.
            sz += Leb128U::from(phi_defns.len() as u64)
                          .send_to(sink) ?;

            for defn in phi_defns {
                let id: InstrId = (*defn).into();
                sz += Leb128U::from(id.as_u32())
                              .send_to(sink) ?;
            }
        }

        Some(sz)
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

impl fmt::Display for InstrPosn {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Ins@{}", self.0)
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
        write!(f, "[{}]", self.0)
    }
}
