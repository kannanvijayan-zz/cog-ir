
use std::fmt;
use std::mem;
use std::marker::PhantomData;

use crate::ops::{ Operation, Opcode, Op };
use crate::ir_types::IrTypeId;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum CmpKind { Lt = 1, Gt, Le, Ge, Eq, Ne }
impl CmpKind {
    fn is_valid_code(code: u8) -> bool {
        (code >= (CmpKind::Lt as u8))
          && (code <= (CmpKind::Ne as u8))
    }
    unsafe fn from_u8(code: u8) -> CmpKind {
        debug_assert!(Self::is_valid_code(code));
        mem::transmute(code)
    }
    fn into_u8(self) -> u8 { self as u8 }
    fn as_str(&self) -> &'static str {
        match *self {
          CmpKind::Lt => "Lt", CmpKind::Gt => "Gt",
          CmpKind::Le => "Le", CmpKind::Ge => "Ge",
          CmpKind::Eq => "Eq", CmpKind::Ne => "Ne",
        }
    }
}

/** Introduces a comparison instruction. */
#[derive(Clone)]
pub struct CmpOp {
    kind: CmpKind,
    tyid: IrTypeId
}

impl CmpOp {
    pub(crate) fn new(kind: CmpKind, tyid: IrTypeId)
      -> CmpOp
    {
        CmpOp { kind, tyid }
    }
}

impl Operation for CmpOp {
    fn opcode() -> Opcode { Opcode::Cmp }
    fn op(&self) -> Op { Op::Cmp(self.clone()) }
    fn out_type(&self) -> Option<IrTypeId> {
        Some(IrTypeId::Bool)
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
          CmpKind::from_u8(*bytes.get_unchecked(0));
        let tyid =
          IrTypeId::from_u8(*bytes.get_unchecked(1));
        (2, CmpOp::new(kind, tyid))
    }
}

impl fmt::Display for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Cmp{}<{}>",
          self.kind.as_str(), self.tyid.as_str())
    }
}
