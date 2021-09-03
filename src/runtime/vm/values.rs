use std::slice;

use crate::runtime::{MutToUnknown, RefToUnknown, Struct};

pub struct UntaggedValue {
    pub data: Box<[u8]>
}

impl UntaggedValue {
    pub(crate) fn instantiate(structure: &Struct) -> Self {
        let ptr = unsafe { std::alloc::alloc(structure.overall_layout) };
        let len = structure.overall_layout.size();
        let data = unsafe { Box::from_raw(slice::from_raw_parts_mut(ptr, len)) };

        let mut value = UntaggedValue { data };

        // NYEO NOTE: This is a safety thing to enable asserts to work
        // It could be dropped for the no-asserts version
        for i in 0..structure.fields.len() {
            // println!("initializing: {}", i);
            value.mut_field(structure, i).initialize_asserts(structure.fields[i].type_data.rust_type)
        }

        value
    }
}

impl UntaggedValue {
    pub(crate) fn ref_field(&self, structure: &Struct, field: usize) -> RefToUnknown<'_> {
        let field = &structure.fields[field];
        let offset = field.offset;
        let len = field.type_data.layout.size();
        RefToUnknown::from(&self.data[offset..offset + len])
    }

    pub(crate) fn mut_field(&mut self, structure: &Struct, field: usize) -> MutToUnknown<'_> {
        let field = &structure.fields[field];
        let offset = field.offset;
        let len = field.type_data.layout.size();
        MutToUnknown::from(&mut self.data[offset..offset + len])
    }
}