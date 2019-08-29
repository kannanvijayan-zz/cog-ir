
use std::fmt::Debug;

/**
 * An IrTypeId is a normal rust enum whose variants
 * identify each IR type.
 */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
pub enum IrTypeId {
    Bool,
    Int32,
    Int64,
    PtrInt
}
impl IrTypeId {
    pub fn as_str(&self) -> &'static str {
        match *self {
            IrTypeId::Bool => "Bool",
            IrTypeId::Int32 => "Int32",
            IrTypeId::Int64 => "Int64",
            IrTypeId::PtrInt => "PtrInt"
        }
    }
}

/**
 * The input type for some operation may be any
 * concrete type, or the `any` type.
 */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
pub enum IrInputTypeId {
    Specific(IrTypeId),
    Any
}

/**
 * The output type for some operation may be any
 * concrete type, or the `void` type.
 */
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
pub enum IrOutputTypeId {
    Specific(IrTypeId),
    Void
}

/**
 * A static trait that describes a closed set
 * of rust types that map to IR types.
 */
pub unsafe trait IrType: Clone + Sized + Debug {
    const ID: IrTypeId;
}
pub unsafe trait IrInputType: Clone + Sized + Debug {
    const INPUT_ID: IrInputTypeId;
}
pub unsafe trait IrOutputType: Clone + Sized + Debug {
    const OUTPUT_ID: IrOutputTypeId;
}

#[derive(Clone, Debug)]
pub struct AnyTy;

#[derive(Clone, Debug)]
pub struct VoidTy;

#[derive(Clone, Debug)]
pub struct BoolTy;

#[derive(Clone, Debug)]
pub struct Int32Ty;

#[derive(Clone, Debug)]
pub struct Int64Ty;

#[derive(Clone, Debug)]
pub struct PtrIntTy;


unsafe impl<T: IrType> IrInputType for T {
    const INPUT_ID: IrInputTypeId =
      IrInputTypeId::Specific(T::ID);
}
unsafe impl<T: IrType> IrOutputType for T {
    const OUTPUT_ID: IrOutputTypeId =
      IrOutputTypeId::Specific(T::ID);
}

unsafe impl IrInputType for AnyTy {
    const INPUT_ID: IrInputTypeId = IrInputTypeId::Any;
}
unsafe impl IrOutputType for VoidTy {
    const OUTPUT_ID: IrOutputTypeId = IrOutputTypeId::Void;
}
unsafe impl IrType for BoolTy {
    const ID: IrTypeId = IrTypeId::Bool;
}
unsafe impl IrType for Int32Ty {
    const ID: IrTypeId = IrTypeId::Int32;
}
unsafe impl IrType for Int64Ty {
    const ID: IrTypeId = IrTypeId::Int64;
}
unsafe impl IrType for PtrIntTy {
    const ID: IrTypeId = IrTypeId::PtrInt;
}
