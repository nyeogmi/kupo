use std::slice;

use moogle::Id;

use crate::codegen::{KStruct, KTypes};
use crate::runtime::{MutToUnknown, RefToUnknown};

pub struct UntaggedValue {
    pub data: Box<[u8]>
}

impl UntaggedValue {
    pub(crate) fn instantiate(types: &KTypes, structure_id: Id<KStruct>) -> Self {
        let structure = types.get_structure(structure_id);
        let ptr = unsafe { std::alloc::alloc(structure.overall_layout) };
        let len = structure.overall_layout.size();
        let data = unsafe { Box::from_raw(slice::from_raw_parts_mut(ptr, len)) };

        let value = UntaggedValue { data };

        // NYEO NOTE: This is a safety thing to enable asserts to work
        // It could be dropped for the no-asserts version
        for i in 0..structure.single_fields.len() {
            // println!("initializing: {}", i);
            let reference = value.ref_single_field(structure, i);
            let type_id = structure.single_fields[i].type_data.type_id;
            reference.initialize_metadata(type_id) 
        }

        value
    }
}

impl UntaggedValue {
    // TODO: Ref ranges of single fields instead of single fields
    pub(crate) fn ref_single_field(&self, structure: &KStruct, single_field: usize) -> RefToUnknown<'_> {
        let field = &structure.single_fields[single_field];
        let offset = field.offset;
        let len = field.type_data.layout.size();
        println!("for {:?}, the single field {:?} has offset {:?} and size {:?}", structure, field, offset, len);
        RefToUnknown::from(&self.data[offset] as *const u8, len)
    }

    pub(crate) fn ref_field(&self, structure: &KStruct, field: usize) -> RefToUnknown<'_> {
        let field = &structure.fields[field];
        let offset = field.offset;
        let len = field.size;
        println!("for {:?}, the full field {:?} has offset {:?} and size {:?}", structure, field, offset, len);
        RefToUnknown::from(&self.data[offset] as *const u8, len)
    }

    pub(crate) fn mut_single_field(&mut self, structure: &KStruct, single_field: usize) -> MutToUnknown<'_> {
        let field = &structure.single_fields[single_field];
        let offset = field.offset;
        let len = field.type_data.layout.size();
        MutToUnknown::from(&mut self.data[offset] as *mut u8, len)
    }

    pub(crate) fn mut_field(&mut self, structure: &KStruct, field: usize) -> MutToUnknown<'_> {
        let field = &structure.fields[field];
        let offset = field.offset;
        let len = field.size;
        println!("for {:?}, the full field {:?} has offset {:?} and size {:?}", structure, field, offset, len);
        MutToUnknown::from(&mut self.data[offset] as *mut u8, len)
    }
}