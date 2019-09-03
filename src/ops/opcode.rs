
use std::mem;

use crate::ops;
use crate::ops::Operation;
use crate::ir_types::{ BoolTy, Int32Ty, Int64Ty, PtrIntTy };

/**
 * An Opcode defines the kind of operation an
 * instruction performs (e.g. a call, add, load, etc.)
 */
#[derive(Clone, Copy, Hash)]
#[derive(PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    // Special instructions.
    Nop = 1, Phi,

    // Introduce const values of various types.
    Const,

    // Comparisons (Lt, Gt, Le, Ge, Eq and Ne)
    Cmp,

    // Binary integer instructions.
    // (Add, Sub, Mul, And, Or, Xor)
    Bini,

    // Terminal instructions.
    Ret, Branch, Jump,
}

pub trait SpecializeOpcode<R> {
    fn op<OP: Operation>(self) -> R;
}

impl Opcode {
    pub const MIN: Opcode = Opcode::Nop;
    pub const MAX: Opcode = Opcode::Jump;

    fn valid_u8(byte: u8) -> bool {
        (byte >= (Self::MIN as u8))
          && (byte <= (Self::MAX as u8))
    }

    pub(crate) unsafe fn from_u8(byte: u8) -> Opcode {
        debug_assert!(Self::valid_u8(byte));
        mem::transmute(byte)
    }

    pub(crate) fn into_u8(self) -> u8 { self as u8 }

    pub(crate) fn specialize<R, S>(self, spec: S) -> R
      where S: SpecializeOpcode<R>
    {
        match self {
          Opcode::Nop => spec.op::<ops::NopOp>(),
          Opcode::Phi => spec.op::<ops::PhiOp>(),
          Opcode::Const => spec.op::<ops::ConstOp>(),
          Opcode::Cmp => spec.op::<ops::CmpOp>(),
          Opcode::Bini => spec.op::<ops::BiniOp>(),
          Opcode::Ret => spec.op::<ops::RetOp>(),
          Opcode::Branch => spec.op::<ops::BranchOp>(),
          Opcode::Jump => spec.op::<ops::JumpOp>()
        }
    }
}

