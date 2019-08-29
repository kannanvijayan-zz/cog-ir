
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
pub struct PhiOp<T: IrType>(u32, PhantomData<T>);

impl<T: IrType> PhiOp<T> {
    pub(crate) fn new(phi_no: u32) -> PhiOp<T> {
        PhiOp(phi_no, Default::default())
    }
}

impl<T: IrType> Operation for PhiOp<T> {
    type Output = T;

    fn opcode(&self) -> Opcode {
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

    fn send_name<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        let s0 = sink.send_slice("Phi") ?;
        let s1 = sink.send_slice(T::ID.as_str()) ?;
        Some(s0 + s1)
    }
}

impl<T: IrType> ByteSerialize for PhiOp<T> {
    fn send_to<S>(&self, sink: &mut S) -> Option<usize>
      where S: ByteSink
    {
        Leb128U::new(self.0 as u64).send_to(sink)
    }

    unsafe fn take_from<S>(src: &mut S) -> (usize, Self)
      where S: ByteSource
    {
        let (bytes, leb) = Leb128U::take_from(src);
        let v = leb.as_u64();
        debug_assert!(v <= (u32::max_value() as u64));
        (bytes, PhiOp::new(v as u32))
    }
}

impl<T: IrType> fmt::Display for PhiOp<T> {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Phi{}", T::ID.as_str())
    }
}
