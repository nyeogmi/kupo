use moogle::Id;

use super::KStruct;

pub enum KType {
    // roughly Box<T> 
    // OwnedPtr(Id<KStruct>),

    // roughly &T
    RefPtr(Id<KStruct>),

    // roughly &mut T
    MutPtr(Id<KStruct>),

    // &mut T, but uninitialized 
    OutPtr(Id<KStruct>),

    // roughly T
    InPlace(Id<KStruct>),
}