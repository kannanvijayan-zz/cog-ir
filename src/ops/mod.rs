
mod opcode;
pub use self::opcode::Opcode;

mod operation;
pub use self::operation::{ Operation, TerminalOperation };

mod bini_op;
pub use self::bini_op::{ BiniOp, BiniKind };

mod branch_op;
pub use self::branch_op::BranchOp;

mod cmp_op;
pub use self::cmp_op::{ CmpOp, CmpKind };

mod const_op;
pub use self::const_op::ConstOp;

mod jump_op;
pub use self::jump_op::JumpOp;

mod nop_op;
pub use self::nop_op::NopOp;

mod phi_op;
pub use self::phi_op::PhiOp;

mod ret_op;
pub use self::ret_op::RetOp;
