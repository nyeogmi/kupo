use std::{alloc::Layout, any::TypeId, fmt};

use crate::{runtime::{RefToUnknown}};

#[derive(Clone)]
pub struct KSingle {
    pub type_id: Option<TypeId>,  // not used for refs to structs generated at runtime
    pub layout: Layout,
    pub clone_callback: Option<fn(RefToUnknown<'_>, RefToUnknown<'_>)>,
    pub debug_callback: fn(RefToUnknown<'_>, &mut fmt::Formatter<'_>),
    pub drop_callback: Option<fn(RefToUnknown<'_>)>,

}
impl std::fmt::Debug for KSingle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeData")
        // .field("rust_type", &self.rust_type)
        .field("layout", &self.layout)
        .field("is_copy", &self.is_copy())
        .field("drop_callback", &self.drop_callback.is_some())
        .finish()
    }
}

impl KSingle {
    pub fn is_copy(&self) -> bool {
        self.clone_callback.is_none()
    }
}