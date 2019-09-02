
use std::fmt;

use crate::ops::{ Operation, Opcode };
use crate::ir_types::{ BoolTy, Int32Ty, Int64Ty };
use crate::leb128;

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

    fn opcode() -> Opcode { Opcode::ConstBool }
    fn num_operands(&self) -> u32 { 0 }

    fn write_to(&self, vec: &mut Vec<u8>) {
        vec.push(self.0 as u8);
    }
    unsafe fn read_from(bytes: &[u8]) -> (usize, Self) {
        debug_assert!(bytes.len() > 0);
        let b = *bytes.get_unchecked(0);
        (0, ConstBoolOp::new(b >= 0))
    }
}

impl Operation for ConstInt32Op {
    type Output = Int32Ty;

    fn opcode() -> Opcode { Opcode::ConstInt32 }
    fn num_operands(&self) -> u32 { 0 }

    fn write_to(&self, vec: &mut Vec<u8>) {
        leb128::write_leb128u(self.0, vec)
    }
    unsafe fn read_from(bytes: &[u8]) -> (usize, Self) {
        let (nb, cv_u64) = leb128::read_leb128u(bytes);
        debug_assert!(cv_u64 <= (u32::max_value() as u64));
        (nb, ConstInt32Op(cv_u64 as u32))
    }
}

impl Operation for ConstInt64Op {
    type Output = Int64Ty;

    fn opcode() -> Opcode { Opcode::ConstInt64 }
    fn num_operands(&self) -> u32 { 0 }

    fn write_to(&self, vec: &mut Vec<u8>) {
        leb128::write_leb128u(self.0, vec);
    }
    unsafe fn read_from(bytes: &[u8]) -> (usize, Self) {
        debug_assert!(bytes.len() > 0);
        let (nb, cv) = leb128::read_leb128u(bytes);
        (nb, ConstInt64Op(cv))
    }
}

impl fmt::Display for ConstBoolOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "ConstBool({})", self.0)
    }
}

impl fmt::Display for ConstInt32Op {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "ConstInt32({})", self.0)
    }
}

impl fmt::Display for ConstInt64Op {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "ConstInt64({})", self.0)
    }
}
