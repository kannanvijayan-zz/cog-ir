
use std::fmt;
use std::marker::PhantomData;

use crate::ops::{ Opcode, Operation };
use crate::ir_types::{ IrType, IrTypeId };

/** Introduces a phi value. */
#[derive(Clone)]
pub struct PhiOp { tyid: IrTypeId }

impl PhiOp {
    pub(crate) fn new(tyid: IrTypeId) -> PhiOp {
        PhiOp { tyid }
    }
}

impl Operation for PhiOp {
    fn opcode() -> Opcode { Opcode::Phi }
    fn out_type(&self) -> Option<IrTypeId> {
        Some(self.tyid)
    }
    fn num_operands(&self) -> u32 { 0 }

    fn write_to(&self, vec: &mut Vec<u8>) {
        vec.push(self.tyid.into_u8());
    }

    unsafe fn read_from(bytes: &[u8]) -> (usize, Self) {
        debug_assert!(bytes.len() >= 1);
        let tyid =
          IrTypeId::from_u8(*bytes.get_unchecked(0));
        (1, PhiOp::new(tyid))
    }
}

impl fmt::Display for PhiOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Phi<{}>", self.tyid.as_str())
    }
}
