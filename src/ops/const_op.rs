
use std::fmt;

use crate::ops::{ Operation, Opcode, Op };
use crate::ir_types::{ IrTypeId, BoolTy, Int32Ty, Int64Ty };
use crate::leb128;

/** Introduces a constant value. */
#[derive(Clone)]
pub enum ConstOp {
    Bool(bool),
    Int32(u32),
    Int64(u64)
}

impl ConstOp {
    pub(crate) fn new_bool(b: bool) -> ConstOp {
        ConstOp::Bool(b)
    }
    pub(crate) fn new_int32(i: u32) -> ConstOp {
        ConstOp::Int32(i)
    }
    pub(crate) fn new_int64(i: u64) -> ConstOp {
        ConstOp::Int64(i)
    }

    fn tyid(&self) -> IrTypeId {
        match self {
          &ConstOp::Bool(_) => IrTypeId::Bool,
          &ConstOp::Int32(_) => IrTypeId::Int32,
          &ConstOp::Int64(_) => IrTypeId::Int64,
        }
    }
}

impl Operation for ConstOp {
    fn opcode() -> Opcode { Opcode::Const }
    fn op(&self) -> Op { Op::Const(self.clone()) }
    fn out_type(&self) -> Option<IrTypeId> {
        Some(self.tyid())
    }
    fn num_operands(&self) -> u32 { 0 }

    fn write_to(&self, vec: &mut Vec<u8>) {
        match self {
          &ConstOp::Bool(b) => {
            vec.extend_from_slice(&[
                IrTypeId::Bool.into_u8(),
                b as u8
            ]);
          }
          &ConstOp::Int32(i) => {
            vec.push(IrTypeId::Int32.into_u8());
            leb128::write_leb128u(i, vec);
          }
          &ConstOp::Int64(i) => {
            vec.push(IrTypeId::Int64.into_u8());
            leb128::write_leb128u(i, vec);
          }
        }
    }
    unsafe fn read_from(bytes: &[u8]) -> (usize, Self) {
        debug_assert!(bytes.len() >= 1);
        let tyid =
          IrTypeId::from_u8(*bytes.get_unchecked(0));

        let rest = bytes.get_unchecked(1..);
        match tyid {
          IrTypeId::Bool => {
            debug_assert!(bytes.len() >= 2);
            let v = *rest.get_unchecked(0);
            (2, ConstOp::Bool(v > 0_u8))
          },
          IrTypeId::Int32 => {
            let (nb, v64) = leb128::read_leb128u(rest);
            debug_assert!(v64 <= (u32::max_value() as u64));
            (1 + nb, ConstOp::Int32(v64 as u32))
          }
          IrTypeId::Int32 => {
            let (nb, v) = leb128::read_leb128u(rest);
            (1 + nb, ConstOp::Int64(v))
          }
          _ => { panic!("Unexpected const type."); }
        }
    }
}

impl fmt::Display for ConstOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        match self {
          &ConstOp::Bool(b) =>
            write!(f, "ConstBool({})", b),
          &ConstOp::Int32(i) =>
            write!(f, "ConstInt32({})", i),
          &ConstOp::Int64(i) =>
            write!(f, "ConstInt64({})", i),
        }
    }
}
