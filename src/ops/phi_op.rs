
use std::fmt;
use std::marker::PhantomData;

use crate::byte_sink::{
    ByteSink, ByteSource, ByteSerialize,
    Leb128U
};
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
    type Output = T;

    fn opcode() -> Opcode {
        match T::ID {
            IrTypeId::Bool => Opcode::PhiBool,
            IrTypeId::Int32 => Opcode::PhiInt32,
            IrTypeId::Int64 => Opcode::PhiInt64,
            IrTypeId::PtrInt => Opcode::PhiPtrInt
        }
    }

    // A phi does not take direct operands.  The
    // inputs are taken from the list of phi inputs
    // given to the end instruction of each directd
    // predecessor block.
    fn num_operands(&self) -> u32 { 0 }
}

impl<T: IrType> ByteSerialize for PhiOp<T> {
    fn send_to<S>(&self, _sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        Some(0)
    }

    unsafe fn take_from<S>(_src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
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
