
use std::fmt;
use std::marker::PhantomData;

use crate::ops::{ Opcode, Operation, TerminalOperation };
use crate::ir_types::{ IrType, IrTypeId, VoidTy };

#[derive(Clone)]
pub struct RetOp<T: IrType>(PhantomData<T>);

impl<T: IrType> RetOp<T> {
    pub(crate) fn new() -> RetOp<T> {
        RetOp(Default::default())
    }
}
impl<T: IrType> TerminalOperation for RetOp<T> {
    fn num_targets(&self) -> u32 { 0 }
}
impl<T: IrType> Operation for RetOp<T> {
    fn opcode() -> Opcode {
        match T::ID {
            IrTypeId::Bool => Opcode::RetBool,
            IrTypeId::Int32 => Opcode::RetInt32,
            IrTypeId::Int64 => Opcode::RetInt64,
            IrTypeId::PtrInt => Opcode::RetPtrInt
        }
    }
    fn out_type(&self) -> Option<IrTypeId> { Some(T::ID) }
    fn num_operands(&self) -> u32 { 1 }

    fn write_to(&self, vec: &mut Vec<u8>) {}

    unsafe fn read_from(_bytes: &[u8]) -> (usize, Self) {
        (0, RetOp::new())
    }
}

impl<T: IrType> fmt::Display for RetOp<T> {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Ret{}", T::ID.as_str())
    }
}
