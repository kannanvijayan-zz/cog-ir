
use std::fmt;

use crate::ops::{
    Opcode, SpecializeOpcode, Operation,
    NopOp, PhiOp,
    ConstOp, CmpOp, BiniOp,
    RetOp, BranchOp, JumpOp
};

#[derive(Clone)]
pub enum Op {
    Nop(NopOp),
    Phi(PhiOp),
    Const(ConstOp),
    Cmp(CmpOp),
    Bini(BiniOp),
    Ret(RetOp),
    Branch(BranchOp),
    Jump(JumpOp)
}

impl Op {
    pub(crate) fn terminal(&self) -> bool {
        match self {
          &Op::Nop(ref op) => false,
          &Op::Phi(ref op) => false,
          &Op::Const(ref op) => false,
          &Op::Cmp(ref op) => false,
          &Op::Bini(ref op) => false,
          &Op::Ret(ref op) => true,
          &Op::Branch(ref op) => true,
          &Op::Jump(ref op) => true,
        }
    }
    pub(crate) fn num_inputs(&self) -> u32 {
        match self {
          &Op::Nop(ref op) => op.num_operands(),
          &Op::Phi(ref op) => op.num_operands(),
          &Op::Const(ref op) => op.num_operands(),
          &Op::Cmp(ref op) => op.num_operands(),
          &Op::Bini(ref op) => op.num_operands(),
          &Op::Ret(ref op) => op.num_operands(),
          &Op::Branch(ref op) => op.num_operands(),
          &Op::Jump(ref op) => op.num_operands(),
        }
    }
    pub(crate) unsafe fn read_from(bytes: &[u8])
      -> (usize, Op)
    {
        // Read an opcode.
        debug_assert!(bytes.len() >= 1);
        let opcode =
          Opcode::from_u8(*bytes.get_unchecked(0));

        let rest = bytes.get_unchecked(1..);
        let (nb, op) =
          opcode.specialize(ReadOperation(rest));
        (1 + nb, op)
    }
}

// Helper struct to specialize on an opcode and
// a op from it.
struct ReadOperation<'a>(&'a [u8]);

impl<'a> SpecializeOpcode<(usize, Op)>
  for ReadOperation<'a>
{
    fn op<OP: Operation>(self) -> (usize, Op) {
        let (nb, typed_op) = unsafe {
          OP::read_from(self.0)
        };
        (nb, typed_op.op())
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        match self {
          &Op::Nop(ref op) => op.fmt(f),
          &Op::Phi(ref op) => op.fmt(f),
          &Op::Const(ref op) => op.fmt(f),
          &Op::Cmp(ref op) => op.fmt(f),
          &Op::Bini(ref op) => op.fmt(f),
          &Op::Ret(ref op) => op.fmt(f),
          &Op::Branch(ref op) => op.fmt(f),
          &Op::Jump(ref op) => op.fmt(f),
        }
    }
}
