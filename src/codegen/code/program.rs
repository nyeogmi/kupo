use moogle::RawPom;

use super::{DefPrototype, ffi::FFI, procedure::Procedure};

pub struct Program {
    pub(crate) def_prototypes: RawPom<DefPrototype>,
    pub(crate) procedures: RawPom<Procedure>,
    pub(crate) ffi: FFI,
}

impl Program {
    pub fn new() -> Program {
        Program { 
            def_prototypes: RawPom::new(), 
            procedures: RawPom::new(), 
            ffi: FFI::new(), 
        }
    }
}