
use std::fmt;
use std::mem;
use std::marker::PhantomData;

use crate::byte_sink::{
    ByteSink, ByteSource, ByteSerialize,
};
use crate::ops::{ Operation, Opcode };
use crate::ir_types::{ IrType, IrTypeId };

/**
 * Integer binops are functions of the form `(T, T) -> T`.
 * Namely, they take two integer inputs of some type T,
 * and return some result taken from that same type T.
 */
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum BiniKind { Add=1, Sub, Mul, And, Or, Xor }
impl BiniKind {
    pub fn is_valid_code(code: u8) -> bool {
        (code >= (BiniKind::Add as u8))
          && (code <= (BiniKind::Xor as u8))
    }
    pub unsafe fn from_u8(code: u8) -> BiniKind {
        debug_assert!(Self::is_valid_code(code));
        mem::transmute(code)
    }
}

/** Binary operation on integers. */
#[derive(Clone)]
pub struct BiniOp<T: IrType>(BiniKind, PhantomData<T>);

impl<T: IrType> BiniOp<T> {
    pub(crate) fn new(kind: BiniKind) -> BiniOp<T> {
        BiniOp(kind, Default::default())
    }
    fn kind(&self) -> BiniKind { self.0 }
}

impl<T: IrType> Operation for BiniOp<T> {
    type Output = T;

    fn opcode(&self) -> Opcode {
        match T::ID {
            IrTypeId::Bool => Opcode::BiniBool,
            IrTypeId::Int32 => Opcode::BiniInt32,
            IrTypeId::Int64 => Opcode::BiniInt64,
            IrTypeId::PtrInt => Opcode::BiniPtrInt
        }
    }

    fn num_operands(&self) -> u32 { 2 }

    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        let s0 = sink.send_slice("Bini") ?;
        let s1 = sink.send_slice(T::ID.as_str()) ?;
        Some(s0 + s1)
    }
}

impl<T: IrType> ByteSerialize for BiniOp<T> {
    fn send_to<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        sink.send_byte(self.0 as u8)
    }

    unsafe fn take_from<S>(src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
        let b = src.take();
        assert!(BiniKind::is_valid_code(b));
        (1, BiniOp::new(BiniKind::from_u8(b)))
    }
}

impl<T: IrType> fmt::Display for BiniOp<T> {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        let s = match self.kind() {
          BiniKind::Add => "Add", BiniKind::Sub => "Sub",
          BiniKind::Mul => "Mul", BiniKind::And => "And",
          BiniKind::Or => "Or", BiniKind::Xor => "Xor",
        };
        write!(f, "Bini{}_{}", s, T::ID.as_str())
    }
}
