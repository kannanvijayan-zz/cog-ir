
use std::fmt;
use std::mem;
use std::marker::PhantomData;

use crate::byte_sink::{
    ByteSink, ByteSource, ByteSerialize
};
use crate::ops::{ Operation, Opcode };
use crate::ir_types::{ IrType, IrTypeId, BoolTy };

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum CmpKind { Lt = 1, Gt, Le, Ge, Eq, Ne }
impl CmpKind {
    pub fn is_valid_code(code: u8) -> bool {
        (code >= (CmpKind::Lt as u8))
          && (code <= (CmpKind::Ne as u8))
    }
    pub unsafe fn from_u8(code: u8) -> CmpKind {
        debug_assert!(Self::is_valid_code(code));
        mem::transmute(code)
    }
    pub fn as_str(&self) -> &'static str {
        match *self {
          CmpKind::Lt => "Lt", CmpKind::Gt => "Gt",
          CmpKind::Le => "Le", CmpKind::Ge => "Ge",
          CmpKind::Eq => "Eq", CmpKind::Ne => "Ne",
        }
    }
}

/** Introduces a comparison instruction. */
#[derive(Clone)]
pub struct CmpOp<T: IrType>(CmpKind, PhantomData<T>);

impl<T: IrType> CmpOp<T> {
    pub(crate) fn new(op: CmpKind) -> CmpOp<T> {
        CmpOp(op, Default::default())
    }
}

impl<T: IrType> Operation for CmpOp<T> {
    type Output = BoolTy;

    fn opcode() -> Opcode {
        match T::ID {
            IrTypeId::Bool => Opcode::CmpBool,
            IrTypeId::Int32 => Opcode::CmpInt32,
            IrTypeId::Int64 => Opcode::CmpInt64,
            IrTypeId::PtrInt => Opcode::CmpPtrInt
        }
    }

    fn num_operands(&self) -> u32 { 2 }
}

impl<T: IrType> ByteSerialize for CmpOp<T> {
    fn send_to<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        sink.send_byte(self.0 as u8)
    }

    unsafe fn take_from<S>(src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
        let b = src.take();
        assert!(CmpKind::is_valid_code(b));
        (1, CmpOp::new(CmpKind::from_u8(b)))
    }
}

impl<T: IrType> fmt::Display for CmpOp<T> {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Cmp{}_{}",
          self.0.as_str(), T::ID.as_str())
    }
}
