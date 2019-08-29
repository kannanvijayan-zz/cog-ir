
use std::fmt;

use crate::byte_sink::{
    ByteSink, ByteSource, ByteSerialize, Leb128U
};
use crate::ops::{ Operation, Opcode };
use crate::ir_types::{ BoolTy, Int32Ty, Int64Ty };

/** Introduces a constant boolean value. */
#[derive(Clone)]
pub struct ConstBoolOp(bool);
impl ConstBoolOp {
    pub(crate) fn new(b: bool) -> ConstBoolOp {
        ConstBoolOp(b)
    }
}

/** Introduces a constant 32-bit integer value. */
#[derive(Clone)]
pub struct ConstInt32Op(u32);
impl ConstInt32Op {
    pub(crate) fn new(i: u32) -> ConstInt32Op {
        ConstInt32Op(i)
    }
}

/** Introduces a constant 64-bit integer value. */
#[derive(Clone)]
pub struct ConstInt64Op(u64);
impl ConstInt64Op {
    pub(crate) fn new(i: u64) -> ConstInt64Op {
        ConstInt64Op(i)
    }
}


impl Operation for ConstBoolOp {
    type Output = BoolTy;

    fn opcode(&self) -> Opcode { Opcode::ConstBool }
    fn num_operands(&self) -> u32 { 0 }

    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        sink.send_slice("ConstBool")
    }
}

impl Operation for ConstInt32Op {
    type Output = Int32Ty;

    fn opcode(&self) -> Opcode { Opcode::ConstInt32 }
    fn num_operands(&self) -> u32 { 0 }

    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        sink.send_slice("ConstInt32")
    }
}

impl Operation for ConstInt64Op {
    type Output = Int64Ty;

    fn opcode(&self) -> Opcode { Opcode::ConstInt64 }
    fn num_operands(&self) -> u32 { 0 }

    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        sink.send_slice("ConstInt64")
    }
}

impl ByteSerialize for ConstBoolOp {
    fn send_to<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        let b = if self.0 { 0_u8 } else { 0x1_u8 };
        sink.send_byte(b)
    }

    unsafe fn take_from<S>(src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
        let b = src.take();
        (1, ConstBoolOp(b == 0u8))
    }
}

impl ByteSerialize for ConstInt32Op {
    fn send_to<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        Leb128U::new(self.0 as u64).send_to(sink)
    }

    unsafe fn take_from<S>(src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
        let (_bs, leb) = Leb128U::take_from(src);
        let v = leb.as_u64();
        assert!(v <= (u32::max_value() as u64));
        (1, ConstInt32Op(v as u32))
    }
}

impl ByteSerialize for ConstInt64Op {
    fn send_to<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        Leb128U::new(self.0 as u64).send_to(sink)
    }

    unsafe fn take_from<S>(src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
        let (_bs, leb) = Leb128U::take_from(src);
        let v = leb.as_u64();
        (1, ConstInt64Op(v))
    }
}

impl fmt::Display for ConstBoolOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "ConstB({})", self.0)
    }
}

impl fmt::Display for ConstInt32Op {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "ConstI32({})", self.0)
    }
}

impl fmt::Display for ConstInt64Op {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "ConstI64({})", self.0)
    }
}
