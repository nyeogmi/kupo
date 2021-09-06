mod bytecode;
use moogle::Id;

pub use bytecode::{Bytecode, GenInstruction, Instruction, Register};

use crate::runtime::{dynamism::{MutToUnknown, RefToUnknown}};

use super::KStruct;

pub struct Program {
    pub(crate) procedures: Vec<Procedure>,
    pub(crate) ffi_ref: Vec<fn(RefToUnknown)>,
    pub(crate) ffi_mut: Vec<fn(RefToUnknown, MutToUnknown)>,
}

pub struct Procedure {
    pub(crate) args: Id<KStruct>,
    pub(crate) locals: Id<KStruct>,
    pub(crate) code: Bytecode,
}