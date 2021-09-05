pub struct Bytecode {
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Register {
    Arg(usize),
    Local(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Instruction {
    // FFI to Rust
    RustCallMut { rust_fn: usize, arg: Register, out: Register},
    RustCallRef { rust_fn: usize, arg: Register},
}