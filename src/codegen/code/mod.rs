mod bytecode;
mod def_prototype;
mod ffi;
mod program;
mod procedure;

pub use bytecode::{Bytecode, GenInstruction, Instruction, Register};
pub use def_prototype::*;
pub use ffi::FFI;
pub use program::Program;
pub use procedure::Procedure;