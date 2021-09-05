use crate::runtime::dynamism::*;
use std::{alloc::Layout, any::{Any, TypeId}, fmt};

// TODO: Track Clone, Debug, and Drop status of types
// Also TODO: Structs should really be made of Structs, not of Ts

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub offset: usize,
    pub type_data: TypeData,
}

pub struct TypeData {
    pub rust_type: TypeId,
    pub layout: Layout,
    pub clone_callback: Option<fn(RefToUnknown<'_>, MutToUnknown<'_>)>,
    pub debug_callback: fn(RefToUnknown<'_>, &mut fmt::Formatter<'_>),
    pub drop_callback: Option<fn(MutToUnknown<'_>)>,
}

impl std::fmt::Debug for TypeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeData")
        .field("rust_type", &self.rust_type)
        .field("layout", &self.layout)
        .field("is_copy", &self.is_copy())
        .field("drop_callback", &self.drop_callback.is_some())
        .finish()
    }
}

impl TypeData {
    pub fn is_copy(&self) -> bool {
        self.clone_callback.is_none()
    }

    pub fn new_copy<T: Any+Copy>(
        debug_callback: fn(RefToUnknown<'_>, &mut fmt::Formatter<'_>),
    ) -> TypeData {
        let rust_type = TypeId::of::<T>();
        TypeData {
            rust_type, 
            layout: Layout::new::<InPlace<T>>(),
            clone_callback: None,
            debug_callback: debug_callback,
            drop_callback: None,
        }
    }
    pub fn new_clone<T: Any>(
        clone_callback: fn(RefToUnknown<'_>, MutToUnknown<'_>),
        debug_callback: fn(RefToUnknown<'_>, &mut fmt::Formatter<'_>),
        drop_callback: Option<fn(MutToUnknown<'_>)>,
    ) -> TypeData {
        let rust_type = TypeId::of::<T>();
        TypeData {
            rust_type, 
            layout: Layout::new::<InPlace<T>>(),
            clone_callback: Some(clone_callback),
            debug_callback: debug_callback,
            drop_callback: drop_callback,
        }
    }
}

pub struct StructBuilder {
    // Type ID, offset, layout
    pub fields: Vec<Field>,
    pub overall_layout: Layout,
}

#[derive(Debug)]
pub struct Struct {
    // Type ID, offset, layout
    pub fields: Vec<Field>,
    pub overall_layout: Layout,
}

impl StructBuilder {
    pub fn new() -> Self {
        StructBuilder {
            fields: vec![],
            overall_layout: Layout::new::<()>(),
        }
    }

    pub fn push(&mut self, name: String, type_data: TypeData) {
        let (new_overall_layout, offset) = self.overall_layout.extend(type_data.layout).unwrap();
        self.fields.push(Field {
            name, 
            offset,
            type_data
        });
        self.overall_layout = new_overall_layout;
    }

    pub fn build(mut self) -> Struct {
        self.overall_layout = self.overall_layout.pad_to_align();
        Struct { fields: self.fields, overall_layout: self.overall_layout }
    }
}