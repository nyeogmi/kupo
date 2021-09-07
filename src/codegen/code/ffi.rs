use moogle::{Id, RawPom};

use crate::runtime::RefToUnknown;

pub struct FFI {
    rust_functions: RawPom<RustFn>,
}

pub struct RustFn { body: fn(RefToUnknown) }

impl FFI {
    pub fn new() -> FFI {
        FFI { 
            rust_functions: RawPom::new(),
        }
    }

    pub fn create_function(&mut self, body: fn(RefToUnknown)) -> Id<RustFn> {
        self.rust_functions.insert(RustFn { body })
    }

    pub fn call(&self, rust_fn: Id<RustFn>, reg: RefToUnknown) {
        (self.rust_functions.get(rust_fn).unwrap().body)(reg)
    }
}