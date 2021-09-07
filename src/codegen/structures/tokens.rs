use moogle::Id;

use super::KStruct;

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