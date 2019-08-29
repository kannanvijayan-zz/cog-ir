
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
    Nop = 1,
    PhiBool, PhiInt32, PhiInt64, PhiPtrInt,

    // Constant instructions.
    ConstBool, ConstInt32, ConstInt64,

    // Comparison operators.
    CmpBool, CmpInt32, CmpInt64, CmpPtrInt,

    // Binary integer instructions.
    BiniBool, BiniInt32, BiniInt64, BiniPtrInt,

    // Terminal instructions.
    RetBool, RetInt32, RetInt64, RetPtrInt,
    Branch, Jump,
}

/**
 * Dispatch handler that gets called to handle
 * some opcode that has been narrowed to a static
 * instruction type.
 */
pub trait SpecializeOpcode<R> {
    fn handle<OP: Operation>(self) -> R;
}

impl Opcode {
    pub const MIN: Opcode = Opcode::Nop;
    pub const MAX: Opcode = Opcode::Jump;

    fn valid_u8(byte: u8) -> bool {
        (byte >= (Self::MIN as u8))
          && (byte <= (Self::MAX as u8))
    }

    pub unsafe fn from_u8(byte: u8) -> Opcode {
        debug_assert!(Self::valid_u8(byte));
        mem::transmute(byte)
    }

    pub fn into_u8(self) -> u8 { self as u8 }

    pub fn specialize<R, H>(self, h: H) -> R
      where H: SpecializeOpcode<R>
    {
        match self {
          Opcode::Nop => {
            h.handle::<ops::NopOp>()
          },
          Opcode::PhiBool => {
            h.handle::<ops::PhiOp<BoolTy>>()
          },
          Opcode::PhiInt32 => {
            h.handle::<ops::PhiOp<Int32Ty>>()
          },
          Opcode::PhiInt64 => {
            h.handle::<ops::PhiOp<Int64Ty>>()
          },
          Opcode::PhiPtrInt => {
            h.handle::<ops::PhiOp<PtrIntTy>>()
          },
          Opcode::ConstBool => {
            h.handle::<ops::ConstBoolOp>()
          },
          Opcode::ConstInt32 => {
            h.handle::<ops::ConstInt32Op>()
          },
          Opcode::ConstInt64 => {
            h.handle::<ops::ConstInt64Op>()
          },
          Opcode::BiniBool => {
            h.handle::<ops::BiniOp<BoolTy>>()
          },
          Opcode::BiniInt32 => {
            h.handle::<ops::BiniOp<Int32Ty>>()
          },
          Opcode::BiniInt64 => {
            h.handle::<ops::BiniOp<Int64Ty>>()
          },
          Opcode::BiniPtrInt => {
            h.handle::<ops::BiniOp<PtrIntTy>>()
          },
          Opcode::CmpBool => {
            h.handle::<ops::CmpOp<BoolTy>>()
          },
          Opcode::CmpInt32 => {
            h.handle::<ops::CmpOp<Int32Ty>>()
          },
          Opcode::CmpInt64 => {
            h.handle::<ops::CmpOp<Int64Ty>>()
          },
          Opcode::CmpPtrInt => {
            h.handle::<ops::CmpOp<PtrIntTy>>()
          },
          Opcode::RetBool => {
            h.handle::<ops::RetOp<BoolTy>>()
          },
          Opcode::RetInt32 => {
            h.handle::<ops::RetOp<Int32Ty>>()
          },
          Opcode::RetInt64 => {
            h.handle::<ops::RetOp<Int64Ty>>()
          },
          Opcode::RetPtrInt => {
            h.handle::<ops::RetOp<PtrIntTy>>()
          },
          Opcode::Branch => {
            h.handle::<ops::BranchOp>()
          },
          Opcode::Jump => {
            h.handle::<ops::JumpOp>()
          },
        }
    }
}

