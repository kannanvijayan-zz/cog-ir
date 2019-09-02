
#[macro_use]
extern crate log;

extern crate env_logger;

mod ir_types;
mod ops;
mod instr;
mod leb128;
mod block;
mod builder;

pub mod api {
    pub use crate::builder::{ Builder, BuildSession };
    pub use crate::ir_types::{
        BoolTy, Int32Ty, Int64Ty, PtrIntTy
    };

    pub fn build<F>(f: F) -> Builder
        where F: for<'x> FnOnce (&mut BuildSession<'x>)
    {
        Builder::build(f)
    }
}
