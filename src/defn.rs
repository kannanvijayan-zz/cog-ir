
use std::fmt;
use std::marker::PhantomData;

use crate::instr::InstrId;
use crate::ir_types::{ IrType, IrOutputType };

/** A definition (just a reference to an instruction). */
#[derive(Clone, Copy, Debug)]
pub struct Defn<'a>(InstrId, PhantomData<&'a ()>);

impl<'a> Defn<'a> {
    pub(crate) fn new(instr_id: InstrId) -> Defn<'a> {
        Defn(instr_id, Default::default())
    }
    pub fn instr_id(&self) -> InstrId { self.0 }
    pub fn as_u32(&self) -> u32 { self.0.as_u32() }
}
impl<'a> Into<InstrId> for Defn<'a> {
    fn into(self) -> InstrId { self.0 }
}
impl<'a> fmt::Display for Defn<'a> {
    fn fmt(&self, f: &mut fmt::Formatter)
      -> Result<(), fmt::Error>
    {
        write!(f, "Def@{}", self.as_u32())
    }
}

/** A typed definition. */
#[derive(Debug)]
pub struct TypedDefn<'a, T: IrOutputType>
    (InstrId, PhantomData<&'a T>);

impl<'a, T: IrOutputType> Clone
  for TypedDefn<'a, T>
{
    fn clone(&self) -> Self {
        TypedDefn(self.0, Default::default())
    }
}
impl<'a, T: IrOutputType> Copy for TypedDefn<'a, T> {}

impl<'a, T: IrOutputType> TypedDefn<'a, T> {
    pub fn new(instr_id: InstrId) -> TypedDefn<'a, T> {
        TypedDefn(instr_id, Default::default())
    }

    pub fn instr_id(&self) -> InstrId { self.0 }
    pub fn untyped_defn(&self) -> Defn<'a> {
        Defn::new(self.instr_id())
    }
    pub fn as_u32(&self) -> u32 { self.0.as_u32() }
}

impl<'a, T> TypedDefn<'a, T>
  where T: IrOutputType + IrType
{
    pub fn cast<U>(&self) -> TypedDefn<'a, U>
      where U: IrOutputType + IrType
    {
        TypedDefn(self.0, Default::default())
    }
}

