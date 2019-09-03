
#[macro_use]
extern crate log;

extern crate env_logger;

mod ir_types;
mod ops;
mod instr;
mod leb128;
mod block;
mod builder;
mod defn;
mod graph;

pub mod api {
    pub use crate::graph::{ Graph, GraphSession };
    pub use crate::builder::{ Builder, BuildSession };
    pub use crate::ir_types::{
        BoolTy, Int32Ty, Int64Ty, PtrIntTy
    };

    pub fn build<F>(f: F) -> Builder
      where F: for<'x> FnOnce (&mut BuildSession<'x>)
    {
        Builder::build(f)
    }

    pub fn graph<R, F>(b: Builder, f: F) -> R
      where F: for <'x> FnOnce (&mut GraphSession<'x>)
                        -> R
    {
        b.into_graph().enter_session(f)
    }
}
