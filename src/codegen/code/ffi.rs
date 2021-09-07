use moogle::{Id, RawPom};

use crate::runtime::{ArgsToUnknown, RefToUnknown};

pub struct FFI {
    rust_functions: RawPom<RustFn>,
}

pub struct RustFn { body: fn(ArgsToUnknown) }

impl FFI {
    pub fn new() -> FFI {
        FFI { 
            rust_functions: RawPom::new(),
        }
    }

    pub fn create_function(&mut self, body: fn(ArgsToUnknown)) -> Id<RustFn> {
        self.rust_functions.insert(RustFn { body })
    }

    pub fn call(&self, rust_fn: Id<RustFn>, reg: ArgsToUnknown) {
        (self.rust_functions.get(rust_fn).unwrap().body)(reg)
    }
}