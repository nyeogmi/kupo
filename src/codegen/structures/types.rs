use std::{alloc::Layout, any::{Any, TypeId}, collections::HashMap, fmt::{self, Formatter}};

use crate::runtime::dynamism::*;
use moogle::*;

use super::{KSingle, KStruct};

pub struct KTypes {
    types: HashMap<TypeId, Id<KStruct>>,
    pub(super) structs: RawPom<KStruct>,
}

impl KTypes {
    pub fn new() -> KTypes {
        KTypes {
            types: HashMap::new(),
            structs: RawPom::new(),
        }
    }

    pub fn get_structure(&self, structure_id: Id<KStruct>) -> &KStruct {
        self.structs.get(structure_id).unwrap()
    }

    pub fn single_copy<T: Any+Copy>(
        &mut self,
        debug_callback: fn(RefToUnknown<'_>, &mut fmt::Formatter<'_>),
    ) -> Id<KStruct> {
        let rust_type = TypeId::of::<T>();

        if let Some(t) = self.types.get(&rust_type) {
            return *t;
        }

        self.promote_single(KSingle {
            type_id: Some(rust_type),
            layout: Layout::new::<InPlace<T>>(),
            clone_callback: None,
            debug_callback: debug_callback,
            drop_callback: None,
        })
    }

    pub fn single_clone<T: Any>(
        &mut self,
        clone_callback: fn(RefToUnknown<'_>, MutToUnknown<'_>),
        debug_callback: fn(RefToUnknown<'_>, &mut fmt::Formatter<'_>),
        drop_callback: Option<fn(MutToUnknown<'_>)>,
    ) -> Id<KStruct> {
        let rust_type = TypeId::of::<T>();

        if let Some(t) = self.types.get(&rust_type) {
            return *t;
        }

        self.promote_single(KSingle {
            type_id: Some(rust_type),
            layout: Layout::new::<InPlace<T>>(),
            clone_callback: Some(clone_callback),
            debug_callback: debug_callback,
            drop_callback: drop_callback,
        })
    }

    fn promote_single(&mut self, single: KSingle) -> Id<KStruct> {
        let layout = single.layout;
        // let id = self.singles.insert(single);

        let type_id = single.type_id;
        let structure = KStruct::wrap(single, layout);
        let struct_id = self.structs.insert(structure);
        if let Some(type_id) = type_id {
            self.types.insert(type_id, struct_id);
        }
        struct_id
    }

    pub(super) fn typedata_for_ref_to(&self, ref_to: &KStruct) -> KSingle {
        // TODO: Separate for ref_mut
        KSingle { 
            type_id: None,
            layout: Layout::new::<&u8>(), 
            clone_callback: None, 
            debug_callback: debug_ptr,  // TODO: Base it on the passed struct?
            drop_callback: None 
        }
    }
}

fn debug_ptr(r: RefToUnknown<'_>, f: &mut Formatter) {
    // TODO: Better output
    f.debug_struct("Pointer {}").finish().unwrap()
}