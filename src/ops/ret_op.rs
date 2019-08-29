
use std::fmt;
use std::marker::PhantomData;

use crate::byte_sink::{
    ByteSink, ByteSource, ByteSerialize
};
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
    fn num_targets(&self) -> usize { 0 }
}
impl<T: IrType> Operation for RetOp<T> {
    type Output = VoidTy;

    fn opcode(&self) -> Opcode {
        match T::ID {
            IrTypeId::Bool => Opcode::RetBool,
            IrTypeId::Int32 => Opcode::RetInt32,
            IrTypeId::Int64 => Opcode::RetInt64,
            IrTypeId::PtrInt => Opcode::RetPtrInt
        }
    }

    fn num_operands(&self) -> u32 { 1 }

    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        let s0 = sink.send_slice("Ret") ?;
        let s1 = sink.send_slice(T::ID.as_str()) ?;
        Some(s0 + s1)
    }
}

impl<T: IrType> ByteSerialize for RetOp<T> {
    fn send_to<S>(&self, _sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        Some(0)
    }

    unsafe fn take_from<S>(_src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
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
