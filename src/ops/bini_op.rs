
use std::fmt;
use std::mem;

use crate::ops::{ Operation, Opcode, Op };
use crate::ir_types::IrTypeId;

/**
 * Integer binops are functions of the form `(T, T) -> T`.
 * Namely, they take two integer inputs of some type T,
 * and return some result taken from that same type T.
 */
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum BiniKind { Add=1, Sub, Mul, And, Or, Xor }
impl BiniKind {
    fn is_valid_code(code: u8) -> bool {
        (code >= (BiniKind::Add as u8))
          && (code <= (BiniKind::Xor as u8))
    }
    unsafe fn from_u8(code: u8) -> BiniKind {
        debug_assert!(Self::is_valid_code(code));
        mem::transmute(code)
    }
    fn into_u8(self) -> u8 { self as u8 }
    fn as_str(self) -> &'static str {
        match self {
          BiniKind::Add => "Add", BiniKind::Sub => "Sub",
          BiniKind::Mul => "Mul", BiniKind::And => "And",
          BiniKind::Or => "Or", BiniKind::Xor => "Xor",
        }
    }
}

/** Binary operation on integers. */
#[derive(Clone)]
pub struct BiniOp {
    kind: BiniKind,
    tyid: IrTypeId
}

impl BiniOp {
    pub(crate) fn new(kind: BiniKind, tyid: IrTypeId)
      -> BiniOp
    {
        BiniOp { kind, tyid }
    }

    fn kind(&self) -> BiniKind { self.kind }
    fn tyid(&self) -> IrTypeId { self.tyid }
}

impl Operation for BiniOp {
    fn opcode() -> Opcode { Opcode::Bini }
    fn op(&self) -> Op { Op::Bini(self.clone()) }
    fn out_type(&self) -> Option<IrTypeId> {
        Some(self.tyid)
    }
    fn num_operands(&self) -> u32 { 2 }

    fn write_to(&self, vec: &mut Vec<u8>) {
        vec.extend_from_slice(&[
            self.kind.into_u8(),
            self.tyid.into_u8()
        ]);
    }
    unsafe fn read_from(bytes: &[u8]) -> (usize, Self) {
        debug_assert!(bytes.len() >= 2);
        let kind =
          BiniKind::from_u8(*bytes.get_unchecked(0));
        let tyid =
          IrTypeId::from_u8(*bytes.get_unchecked(1));
        (2, BiniOp { kind, tyid })
    }
}

impl fmt::Display for BiniOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Bini{}<{}>",
          self.kind.as_str(), self.tyid.as_str())
    }
}
