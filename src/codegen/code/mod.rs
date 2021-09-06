mod bytecode;
mod structure;

pub use bytecode::{Bytecode, GenInstruction, Instruction, Register};
pub use structure::{Struct, StructBuilder, TypeData};

use crate::runtime::{dynamism::{MutToUnknown, RefToUnknown}};

pub struct Program {
    pub(crate) procedures: Vec<Procedure>,
    pub(crate) ffi_ref: Vec<fn(RefToUnknown)>,
    pub(crate) ffi_mut: Vec<fn(RefToUnknown, MutToUnknown)>,
}

pub struct Procedure {
    pub(crate) args: Struct,
    pub(crate) locals: Struct,
    pub(crate) code: Bytecode,
}