use moogle::Id;

use crate::codegen::{KStruct, KStructBuilder, KType, KTypes};

pub struct DefPrototype {
    args: Vec<DefProtoArg>,
    args_struct: Id<KStruct>,
}

pub struct DefPrototypeBuilder {
    args: Vec<DefProtoArg>,
}

struct DefProtoArg {
    name: String,
    arg_type: DefProtoArgType,
}

pub enum DefProtoArgType {
    Ref(Id<KStruct>),
    Mut(Id<KStruct>),
    Out(Id<KStruct>),
}


impl DefPrototypeBuilder {
    pub fn new() -> Self {
        DefPrototypeBuilder { args: vec![] }
    }

    pub fn add_arg(&mut self, name: String, arg_type: DefProtoArgType) {
        if self.args.iter().position(|a| a.name == name).is_some() {
            panic!("can't have multiple args of the same name");
        }
        self.args.push(DefProtoArg { name, arg_type })
    }

    pub fn build(self, t: &mut KTypes) -> DefPrototype {
        let mut builder = KStructBuilder::new();
        for arg in self.args.iter() {
            let name = arg.name.clone();
            match arg.arg_type {
                DefProtoArgType::Ref(r) => { builder.push(t, name, KType::RefPtr(r)) }
                DefProtoArgType::Mut(m) => { builder.push(t, name, KType::MutPtr(m)) }
                DefProtoArgType::Out(o) => { builder.push(t, name, KType::OutPtr(o)) }
            }
        }
        let args_struct = builder.build(t);
        DefPrototype { args: self.args, args_struct }
    }
}