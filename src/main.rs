#![feature(type_alias_impl_trait)]

mod codegen;
mod runtime;
mod frontend;

use std::cell::{Cell, RefCell};
use std::fmt::Debug;

use moogle::RawPom;

use crate::codegen::*;
use crate::runtime::{UntaggedValue, VM};
use crate::frontend::parse_module;

fn main_old() {
    let mut types = KTypes::new();
    let string = types.single_copy::<&'static str>(
        |ptr, dbg| 
        ptr.cast::<&'static str>().get().fmt(dbg).unwrap(),
    );

    let mut args = KStructBuilder::new();
    args.push(&mut types, "a1".to_string(), KType::InPlace(string));
    let args = args.build(&mut types);

    let locals = KStructBuilder::new().build(&mut types);

    let mut ffi = FFI::new();
    let rust_fn = ffi.create_function(|arg| {
        let arg2 = arg.cast::<&'static str>();
        let a = arg2.get();
        println!("Program sez: {}", a);
    });

    let def_prototypes = RawPom::new();
    let mut procedures = RawPom::new();

    let proc = procedures.insert(
        Procedure{
            args, locals,
            code: Bytecode { instructions: vec![
                Instruction::RustCall { rust_fn, out: Register::Arg(0), arg: Register::Arg(0) },
            ] },
        }
    );

    let program = codegen::Program {
        def_prototypes, procedures, ffi,
    };

    let mut untagged = UntaggedValue::instantiate(&types, args);
    untagged.ref_single_field(
        types.get_structure(args), 
        0
    ).cast::<&'static str>().initialize("Hello, world!");

    let vm = VM::new(program, types);
    vm.call(proc, untagged);
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