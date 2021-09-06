use moogle::Id;

use crate::codegen::KStruct;

use super::Bytecode;

pub struct Procedure {
    pub(crate) args: Id<KStruct>,
    pub(crate) locals: Id<KStruct>,
    pub(crate) code: Bytecode,
}