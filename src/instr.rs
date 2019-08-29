
use std::fmt;

use crate::ops::{ Operation, TerminalOperation };
use crate::block::BlockId;
use crate::byte_sink::{ ByteSink, ByteSerialize, Leb128U };

/**
 * An instruction object wraps an operation, along
 * with the input definitions it takes.
 */
pub trait Instr {
    type Op: Operation;
    type Def: Copy + Into<InstrNo>;

    fn op(&self) -> &Self::Op;
    fn inputs(&self) -> &[Self::Def];

    fn send_instr<S: ByteSink>(&self, sink: &mut S)
      -> Option<usize>
    {
        // Encode the opcode for the instruction.
        let opcode = self.op().opcode();
        let mut sz = sink.send_byte(opcode.into_u8()) ?;

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
 * An end-instruction wraps a terminal operation,
 * its inputs, target references and their phi inputs.
 */
pub trait EndInstr {
    type Op: TerminalOperation + Operation;
    type Def: Copy + Into<InstrNo>;
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
                let id: InstrNo = (*defn).into();
                sz += Leb128U::from(id.as_u32())
                              .send_to(sink) ?;
                debug!("    => phi defn {}", id);
            }

        }

        Some(sz)
    }
}

pub struct InstrObj<'a, OP, DEF>(&'a OP, &'a [DEF])
  where OP: Operation,
        DEF: Copy + Into<InstrNo>;

pub struct EndInstrObj<'a, OP, DEF, BLK>(
    &'a OP, &'a [DEF], &'a [(BLK, &'a [DEF])])
  where OP: TerminalOperation + Operation,
        DEF: Copy + Into<InstrNo>,
        BLK: Copy + Into<BlockId>;

impl<'a, OP, DEF> Instr for InstrObj<'a, OP, DEF>
  where OP: Operation,
        DEF: Copy + Into<InstrNo>
{
    type Op = OP;
    type Def = DEF;

    fn op(&self) -> &Self::Op { &self.0 }
    fn inputs(&self) -> &[Self::Def] { &self.1 }
}

impl<'a, OP, DEF, BLK> EndInstr
  for EndInstrObj<'a, OP, DEF, BLK>
  where OP: TerminalOperation + Operation,
        DEF: Copy + Into<InstrNo>,
        BLK: Copy + Into<BlockId>
{
    type Op = OP;
    type Def = DEF;
    type Blk = BLK;

    fn op(&self) -> &Self::Op { &self.0 }
    fn inputs(&self) -> &[Self::Def] { &self.1 }
    fn targets(&self) -> &[(Self::Blk, &[Self::Def])] {
        &self.2
    }
}

impl<'a, OP, DEF> InstrObj<'a, OP, DEF>
  where OP: Operation,
        DEF: Copy + Into<InstrNo>
{
    pub(crate) fn new(op: &'a OP, inputs: &'a [DEF])
      -> InstrObj<'a, OP, DEF>
    {
        InstrObj(op, inputs)
    }
}

impl<'a, OP, DEF, BLK> EndInstrObj<'a, OP, DEF, BLK>
  where OP: TerminalOperation + Operation,
        DEF: Copy + Into<InstrNo>,
        BLK: Copy + Into<BlockId>
{
    pub(crate) fn new(op: &'a OP, inputs: &'a [DEF],
                      targets: &'a [(BLK, &'a [DEF])])
      -> EndInstrObj<'a, OP, DEF, BLK>
    {
        EndInstrObj(op, inputs, targets)
    }
}

/**
 * The offset of an instruction in the instruction stream.
 * Serves as the canonical id for an instruction.
 */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrPosn(u32);
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
        write!(f, "Ins@[{}]", self.0)
    }
}

/**
 * The numbering of an instruction in the global ordering
 * of emitted instructions.
 */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct InstrNo(u32);
impl InstrNo {
    const INVALID_VALUE: u32 = u32::max_value();

    pub(crate) fn new(num: u32) -> InstrNo {
        debug_assert!(num != Self::INVALID_VALUE);
        InstrNo(num)
    }
    pub(crate) fn as_u32(&self) -> u32 { self.0 }

    pub(crate) fn invalid() -> InstrNo {
        InstrNo(Self::INVALID_VALUE)
    }

    pub(crate) fn take(&mut self) -> InstrNo {
        let r = *self;
        self.0 += 1;
        r
    }
}
impl fmt::Display for InstrNo {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Ins#({})", self.0)
    }
}
