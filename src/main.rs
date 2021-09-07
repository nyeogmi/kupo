#![feature(type_alias_impl_trait)]

mod codegen;
mod runtime;
mod frontend;

use std::fmt::Debug;
use std::mem::size_of;

use moogle::RawPom;

use crate::codegen::*;
use crate::runtime::{InPlaceCell, UntaggedValue, VM};
use crate::frontend::parse_module;

// TODO: Make sure my calling convention is actually followed holy shit!!!!!
fn main_old() {
    let mut types = KTypes::new();
    let string = types.single_copy::<char>(
        |ptr, dbg| 
        ptr.cast_copy::<char>().get().fmt(dbg).unwrap(),
    );

    let mut args = KStructBuilder::new();
    args.push(&mut types, "a1".to_string(), KType::RefPtr(string));
    args.push(&mut types, "a2".to_string(), KType::RefPtr(string));
    let args = args.build(&mut types);

    let mut rust_fn_args = KStructBuilder::new();
    rust_fn_args.push(&mut types, "word1".to_string(), KType::InPlace(string));
    rust_fn_args.push(&mut types, "word2".to_string(), KType::InPlace(string));
    let rust_fn_args = rust_fn_args.build(&mut types);

    let mut locals = KStructBuilder::new();
    locals.push(&mut types, "call".to_string(), KType::InPlace(rust_fn_args));
    let locals = locals.build(&mut types);

    println!("string is: {:?}", string);
    println!("args is: {:?}", args);
    println!("rust_fn_args is: {:?}", rust_fn_args);
    println!("locals is: {:?}", locals);

    let mut ffi = FFI::new();
    let rust_fn = ffi.create_function(|args| {
        let arg1 = args.arg(0);
        let arg2 = args.arg(1);
        let word1 = arg1.cast_copy::<char>();
        let word2 = arg2.cast_copy::<char>();
        println!("Program sez: {:?} {:?} {} {}", arg1, arg2, word1.get(), word2.get());
    });

    let def_prototypes = RawPom::new();
    let mut procedures = RawPom::new();

    let proc = procedures.insert(
        Procedure{
            args, locals,
            code: Bytecode { instructions: vec![
                Instruction::MakeRef { field_out: 0, out: Register::Local(0), arg: Register::Arg(1) },
                Instruction::MakeRef { field_out: 1, out: Register::Local(0), arg: Register::Arg(0) },
                Instruction::RustCall { rust_fn, out: Register::Local(0), arg: Register::Local(0) },
            ] },
        }
    );

    let program = codegen::Program {
        def_prototypes, procedures, ffi,
    };

    let untagged = UntaggedValue::instantiate(&types, args);
    println!("{:?}", types.get_structure(rust_fn_args));
    untagged.ref_single_field( types.get_structure(args), 0).cast_copy::<char>().initialize('k');
    untagged.ref_single_field( types.get_structure(args), 1).cast_copy::<char>().initialize('w');

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