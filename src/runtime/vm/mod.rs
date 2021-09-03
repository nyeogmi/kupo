mod values;

use crate::runtime::Program;

pub use self::values::UntaggedValue;

use super::{MutToUnknown, Procedure, RefToUnknown, Register};

pub struct VM {
    program: Program
}

impl VM {
    pub fn new(program: Program) -> VM {
        VM {program}
    }

    pub fn call(&self, procedure: usize, args: UntaggedValue) {
        let proc = &self.program.procedures[procedure];
        let mut frame = Frame { 
            procedure, 
            ip: 0,
            args, 
            locals: UntaggedValue::instantiate(&proc.locals) ,
        };
        // interpret(proc.code
        self.interpret(&mut frame);
    }

    fn interpret(&self, frame: &mut Frame) -> () {
        let proc = &self.program.procedures[frame.procedure];

        while frame.ip < proc.code.instructions.len() {
            match proc.code.instructions[frame.ip] {
                super::Instruction::RustCallMut { rust_fn, arg, out } => {
                    let (rr, mr) = frame.mut_register2(proc, arg, out);
                    (self.program.ffi_mut[rust_fn])(rr.downgrade(), mr)
                }
                super::Instruction::RustCallRef { rust_fn, arg } => {
                    (self.program.ffi_ref[rust_fn])(frame.ref_register(proc, arg))
                }
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
    fn ref_register<'a>(&'a self, proc: &'a Procedure, reg: Register) -> RefToUnknown<'a> {
        match reg {
            Register::Arg(a) => { self.args.ref_field(&proc.args, a) }
            Register::Local(l) => { self.locals.ref_field(&proc.locals, l) }
        }
    }

    fn mut_register<'a>(&'a mut self, proc: &'a Procedure, reg: Register) -> MutToUnknown<'a> {
        match reg {
            Register::Arg(a) => { self.args.mut_field(&proc.args, a) }
            Register::Local(l) => { self.locals.mut_field(&proc.locals, l) }
        }
    }

    fn mut_register2<'a>(&'a mut self, proc: &'a Procedure, m1: Register, m2: Register) -> (MutToUnknown<'a>, MutToUnknown<'a>) {
        assert_ne!(m1, m2);
        let ptr = {self as *mut Self};
        let m1_part = unsafe{&mut *ptr}.mut_register(proc, m1);
        let m2_part = unsafe{&mut *ptr}.mut_register(proc, m2);
        (m1_part, m2_part)
    }
}