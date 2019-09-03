
use crate::ops::{
    Opcode, SpecializeOpcode, Operation,
    NopOp, PhiOp,
    ConstOp, CmpOp, BiniOp,
    RetOp, BranchOp, JumpOp
};

#[derive(Clone)]
pub enum Op {
    Nop(NopOp),
    Phi(PhiOp),
    Const(ConstOp),
    Cmp(CmpOp),
    Bini(BiniOp),
    Ret(RetOp),
    Branch(BranchOp),
    Jump(JumpOp)
}

impl Op {
    pub(crate) unsafe fn read_from(bytes: &[u8])
      -> (usize, Op)
    {
        // Read an opcode.
        debug_assert!(bytes.len() >= 1);
        let opcode =
          Opcode::from_u8(*bytes.get_unchecked(0));

        let rest = bytes.get_unchecked(1..);
        opcode.specialize(ReadOperation(rest))
    }
}

// Helper struct to specialize on an opcode and
// a op from it.
struct ReadOperation<'a>(&'a [u8]);

impl<'a> SpecializeOpcode<(usize, Op)>
  for ReadOperation<'a>
{
    fn op<OP: Operation>(self) -> (usize, Op) {
        let (nb, typed_op) = unsafe {
          OP::read_from(self.0)
        };
        (nb, typed_op.op())
    }
}
