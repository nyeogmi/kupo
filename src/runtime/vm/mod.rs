mod values;

use moogle::Id;

use crate::codegen::{GenInstruction, KTypes, Procedure, Program, Register};

pub use self::values::UntaggedValue;

use super::{RefToUnknown};

pub struct VM {
    program: Program,
    types: KTypes,
}

impl VM {
    pub fn new(program: Program, types: KTypes) -> VM {
        VM {program, types}
    }

    pub fn call(&self, procedure: Id<Procedure>, args: UntaggedValue) {
        let proc = &self.program.procedures.get(procedure).unwrap();
        let mut frame = Frame { 
            procedure, 
            ip: 0,
            args, 
            locals: UntaggedValue::instantiate(&self.types, proc.locals) ,
        };
        // interpret(proc.code
        self.interpret(&mut frame);
    }

    fn interpret(&self, frame: &mut Frame) -> () {
        let proc = &self.program.procedures.get(frame.procedure).unwrap();

        while frame.ip < proc.code.instructions.len() {
            match proc.code.instructions[frame.ip] {
                GenInstruction::RustCall{ rust_fn, out, arg } => {
                    assert!(out == arg); // TODO: Disable this for no asserts mode
                    let reg = frame.ref_register(&self.types, proc, arg);
                    self.program.ffi.call(rust_fn, reg);
                }
                GenInstruction::Jump { label } => {
                    frame.ip = label
                }
                GenInstruction::Move { out, arg } => todo!(),
                GenInstruction::Return {  } => todo!(),
            }
            frame.ip += 1;
        }
    }
}

pub struct Frame {
    procedure: Id<Procedure>,
    ip: usize,
    args: UntaggedValue,
    locals: UntaggedValue,
}

impl Frame {
    fn ref_register<'a>(&'a self, types: &KTypes, proc: &'a Procedure, reg: Register) -> RefToUnknown<'a> {
        match reg {
            Register::Arg(a) => { 
                let args = types.get_structure(proc.args);
                self.args.ref_single_field(&args, a) 
            }
            Register::Local(l) => { 
                let locals = types.get_structure(proc.locals);
                self.locals.ref_single_field(&locals, l) 
            }
        }
    }
}