use moogle::Id;

use super::{KStruct, KTypes};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum KType {
    // roughly Box<T> 
    // OwnedPtr(Id<KStruct>),

    // roughly &T
    RefPtr(Id<KStruct>),

    // &T, but uninitialized 
    OutPtr(Id<KStruct>),

    // roughly T
    InPlace(Id<KStruct>),
}

impl KType {
    pub fn is_copy(&self, types: &KTypes) -> bool {
        match self {
            KType::RefPtr(_) => true,
            KType::OutPtr(_) => true,  // todo: is this true?
            KType::InPlace(s) => {
                types.get_structure(*s).is_copy()
            }
        }
    }
}