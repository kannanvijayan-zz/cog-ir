
use std::fmt;
use std::marker::PhantomData;

use crate::ops::{ Opcode, Operation, TerminalOperation };
use crate::ir_types::{ IrType, IrTypeId, VoidTy };

#[derive(Clone)]
pub struct RetOp { tyid: IrTypeId }

impl RetOp {
    pub(crate) fn new(tyid: IrTypeId) -> RetOp {
        RetOp { tyid }
    }
}
impl TerminalOperation for RetOp {
    fn num_targets(&self) -> u32 { 0 }
}
impl Operation for RetOp {
    fn opcode() -> Opcode { Opcode::Ret }
    fn out_type(&self) -> Option<IrTypeId> {
      Some(self.tyid)
    }
    fn num_operands(&self) -> u32 { 1 }

    fn write_to(&self, vec: &mut Vec<u8>) {
        vec.push(self.tyid.into_u8());
    }

    unsafe fn read_from(bytes: &[u8]) -> (usize, Self) {
        debug_assert!(bytes.len() >= 1);
        let tyid =
          IrTypeId::from_u8(*bytes.get_unchecked(0));
        (1, RetOp::new(tyid))
    }
}

impl fmt::Display for RetOp {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Ret<{}>", self.tyid.as_str())
    }
}
