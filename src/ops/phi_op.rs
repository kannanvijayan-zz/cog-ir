
use std::fmt;
use std::marker::PhantomData;

use crate::ops::{ Opcode, Operation };
use crate::ir_types::{ IrType, IrTypeId };

/** Introduces a phi value. */
#[derive(Clone)]
pub struct PhiOp<T: IrType>(PhantomData<T>);

impl<T: IrType> PhiOp<T> {
    pub(crate) fn new() -> PhiOp<T> {
        PhiOp(Default::default())
    }
}

impl<T: IrType> Operation for PhiOp<T> {
    fn opcode() -> Opcode {
        match T::ID {
            IrTypeId::Bool => Opcode::PhiBool,
            IrTypeId::Int32 => Opcode::PhiInt32,
            IrTypeId::Int64 => Opcode::PhiInt64,
            IrTypeId::PtrInt => Opcode::PhiPtrInt
        }
    }
    fn out_type(&self) -> Option<IrTypeId> { Some(T::ID) }

    // A phi does not take direct operands.  The
    // inputs are taken from the list of phi inputs
    // given to the end instruction of each directd
    // predecessor block.
    fn num_operands(&self) -> u32 { 0 }

    fn write_to(&self, vec: &mut Vec<u8>) {}

    unsafe fn read_from(_bytes: &[u8]) -> (usize, Self) {
        (0, PhiOp::new())
    }
}

impl<T: IrType> fmt::Display for PhiOp<T> {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Phi{}", T::ID.as_str())
    }
}
