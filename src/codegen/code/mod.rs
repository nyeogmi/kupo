mod bytecode;
mod ffi;
mod program;
mod procedure;
mod def_prototype;

pub use bytecode::{Bytecode, GenInstruction, Instruction, Register};
pub use program::Program;
pub use def_prototype::*;