use std::alloc::Layout;

use moogle::Id;

use super::{KSingle, KType, KTypes};

pub struct KStruct {
    pub fields: Vec<KField>,
    pub single_fields: Vec<KSingleField>,
    pub overall_layout: Layout,
}

pub struct KStructBuilder {
    pub fields: Vec<KField>,
    pub single_fields: Vec<KSingleField>,
    pub overall_layout: Layout,
}

pub struct KField {
    pub name: String,
    pub offset: usize,
    pub practical_type: KType,
}

#[derive(Clone, Debug)]
pub struct KSingleField {
    pub offset: usize,

    // NYEO NOTE: This is an indirection
    // TODO: Remove this indirection and clone it instead
    // pub type_data: Id<KSingle>, 
    pub type_data: KSingle, 
}

impl KStruct {
    pub(super) fn wrap(single: KSingle, layout: Layout) -> KStruct {
        KStruct {
            fields: vec![],  // no visible fields
            single_fields: vec![KSingleField { offset: 0, type_data: single }],
            overall_layout: layout
        }
    }
}

impl KStructBuilder {
    pub fn new() -> KStructBuilder {
        KStructBuilder {
            fields: vec![], 
            single_fields: vec![],
            overall_layout: Layout::new::<()>(),
        }
    }

    pub fn push(&mut self, types: &mut KTypes, name: String, ty: KType) {
        match ty {
            KType::RefPtr(t) |
            KType::MutPtr(t) |
            KType::OutPtr(t) => {
                let real_struct = types.structs.get(t).unwrap();
                let typedata = types.typedata_for_ref_to(real_struct);
                let (new_overall_layout, offset) = 
                    self.overall_layout.extend(typedata.layout).unwrap();

                self.single_fields.push(KSingleField {
                    offset: offset,
                    type_data: typedata,
                });
                self.fields.push(KField { name, offset, practical_type: ty });
                self.overall_layout = new_overall_layout;
            }
            KType::InPlace(t) => {
                let real_struct = types.structs.get(t).unwrap();

                let (new_overall_layout, offset) = 
                    self.overall_layout.extend(real_struct.overall_layout).unwrap();

                for field in &real_struct.single_fields {
                    self.single_fields.push(KSingleField { 
                        offset: offset + field.offset, 
                        type_data: field.type_data.clone()
                    })
                }
                self.fields.push(KField { 
                    name, 
                    offset, 
                    practical_type: KType::InPlace(t)
                });

                self.overall_layout = new_overall_layout;
            }
        }
    }

    pub fn build(mut self, types: &mut KTypes) -> Id<KStruct> {
        types.structs.insert(KStruct { 
            fields: self.fields, 
            single_fields: self.single_fields, 
            overall_layout: self.overall_layout 
        })
    }
}