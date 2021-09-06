mod values;

use crate::codegen::{GenInstruction, KTypes, Procedure, Program, Register};

pub use self::values::UntaggedValue;

use super::{MutToUnknown, RefToUnknown};

pub struct VM {
    program: Program,
    types: KTypes,
}

impl VM {
    pub fn new(program: Program, types: KTypes) -> VM {
        VM {program, types}
    }

    pub fn call(&self, procedure: usize, args: UntaggedValue) {
        let proc = &self.program.procedures[procedure];
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
        let proc = &self.program.procedures[frame.procedure];

        while frame.ip < proc.code.instructions.len() {
            match proc.code.instructions[frame.ip] {
                GenInstruction::RustCallMut { rust_fn, arg, out } => {
                    let (rr, mr) = frame.mut_register2(&self.types, proc, arg, out);
                    (self.program.ffi_mut[rust_fn])(rr.downgrade(), mr)
                }
                GenInstruction::RustCallRef { rust_fn, arg } => {
                    (self.program.ffi_ref[rust_fn])(frame.ref_register(&self.types, proc, arg))
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
    procedure: usize,
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

    fn mut_register<'a>(&'a mut self, types: &KTypes, proc: &'a Procedure, reg: Register) -> MutToUnknown<'a> {
        match reg {
            Register::Arg(a) => { 
                let args = types.get_structure(proc.args);
                self.args.mut_single_field(&args, a) 
            }
            Register::Local(l) => { 
                let locals = types.get_structure(proc.locals);
                self.locals.mut_single_field(&locals, l) 
            }
        }
    }

    fn mut_register2<'a>(&'a mut self, types: &KTypes, proc: &'a Procedure, m1: Register, m2: Register) -> (MutToUnknown<'a>, MutToUnknown<'a>) {
        assert_ne!(m1, m2);
        let ptr = {self as *mut Self};
        let m1_part = unsafe{&mut *ptr}.mut_register(types, proc, m1);
        let m2_part = unsafe{&mut *ptr}.mut_register(types, proc, m2);
        (m1_part, m2_part)
    }
}