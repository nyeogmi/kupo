#![feature(type_alias_impl_trait)]

mod runtime;
mod frontend;

use std::fmt::Debug;

use crate::runtime::{Bytecode, Instruction, Procedure, Register, TypeData, UntaggedValue, VM};
use crate::frontend::parse_module;

fn main_old() {
    let mut args = runtime::StructBuilder::new();
    args.push("a1".to_string(), TypeData::new_copy::<&'static str>(
        |ptr, dbg| ptr.cast::<&'static str>().get().fmt(dbg).unwrap(),
    ));
    let args = args.build();
    let locals = runtime::StructBuilder::new().build();

    let program = runtime::Program {
        procedures: vec![
            Procedure{
                args, locals,
                code: Bytecode { instructions: vec![
                    Instruction::RustCallRef { rust_fn: 0, arg: Register::Arg(0) },
                ] },
            }
        ],
        ffi_ref: vec![
            |arg| {
                let arg2 = arg.cast::<&'static str>();
                println!("Program sez: {}", arg2.get());
            }
        ],
        ffi_mut: vec![],
    };

    let args = &program.procedures[0].args;
    let mut untagged = UntaggedValue::instantiate(args);
    untagged.mut_field(args,0).cast::<&'static str>().initialize("Hello, world!");

    let vm = VM::new(program);
    vm.call(0, untagged);
}

fn main() {
    main_old();
    println!("{:?}", parse_module("
    view [@x NPC] in lonely_vampire {}

    def main() [] {
        print('Test!')
    }
    "));
    println!("{:?}", parse_module("
    def main() {
        print(1+2*3)
    }
    "));
    println!("{:?}", parse_module("
    def main(@x) {
        print(1+2*3
    }
    "));
}