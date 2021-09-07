mod values;

use moogle::Id;

use crate::codegen::{GenInstruction, KStruct, KType, KTypes, Procedure, Program, Register};

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
                    let (called_with, reg) = frame.ref_register_struct(&self.types, proc, arg);
                    println!("calling: with {:?}", called_with);
                    self.program.ffi.call(rust_fn, reg.to_args(called_with));
                }
                GenInstruction::Jump { label } => {
                    frame.ip = label
                }
                GenInstruction::Return {  } => todo!(),

                GenInstruction::Copy { out, arg } => todo!(),
                GenInstruction::CopyField { field_out, out, field_arg, arg } => {
                    let ((out_type, out_ref), (arg_type, arg_ref)) = 
                        frame.mut_register2_struct(&self.types, proc, out, arg);

                    let (out_offset, out_ptype, out_nbytes) = match out_type {
                        KType::RefPtr(_) => panic!("doesn't work for ref ptrs"),
                        KType::OutPtr(_) => panic!("doesn't work for out ptrs"),
                        KType::InPlace(s) => {
                            let t = &self.types.get_structure(s).fields[field_out];
                            (t.offset, t.practical_type, t.size)
                        }
                    };

                    let (arg_offset, arg_ptype, arg_nbytes) = match arg_type {
                        KType::RefPtr(_) => panic!("doesn't work for ref ptrs"),
                        KType::OutPtr(_) => panic!("doesn't work for out ptrs"),
                        KType::InPlace(s) => {
                            let t = &self.types.get_structure(s).fields[field_arg];
                            (t.offset, t.practical_type, t.size)
                        }
                    };

                    println!("out type: {:?}; arg_type: {:?}", out_type, arg_type);
                    assert!(out_ptype == arg_ptype);
                    assert!(out_nbytes == arg_nbytes);
                    assert!(out_ptype.is_copy(&self.types));

                    unsafe { out_ref.copy_from(out_offset, arg_offset, out_nbytes, arg_ref.downgrade()) };
                }

                GenInstruction::MakeRef { field_out, out, arg } => {
                    let ((out_type, out_ref), (arg_type, arg_ref)) = 
                        frame.mut_register2_struct(&self.types, proc, out, arg);

                    println!("!!! want to write {:?}[{:?}].{:?} = {:?}[{:?}]", out, out_type, field_out, arg, arg_type);

                    let (out_offset, out_ptype, out_nbytes) = match out_type {
                        KType::RefPtr(_) => panic!("doesn't work for ref ptrs"),
                        KType::OutPtr(_) => panic!("doesn't work for out ptrs"),
                        KType::InPlace(s) => {
                            let t = &self.types.get_structure(s).fields[field_out];
                            (t.offset, t.practical_type, t.size)
                        }
                    };

                    let (arg_underlying_type, arg_underlying_ref) = match arg_type {
                        KType::RefPtr(underlying_type) | 
                        KType::OutPtr(underlying_type) => { 
                            let utype_data = self.types.get_structure(underlying_type);
                            let ipc = arg_ref.downgrade().cast_copy::<&u8>().get();
                            (underlying_type, RefToUnknown::from(ipc as *const u8, utype_data.overall_layout.size()))
                        },
                        KType::InPlace(ut) => { (ut, arg_ref.downgrade()) }
                    };

                    // TODO: Appropriate asserts
                    /*
                    println!("out type: {:?}; arg_type: {:?}", out_type, arg_type);
                    assert!(out_ptype == KType::RefPtr(arg_underlying_type));
                    assert!(out_nbytes == arg_nbytes);
                    assert!(out_ptype.is_copy(&self.types));
                    */

                    unsafe { out_ref.write_ref(out_offset, arg_underlying_ref) };

                }
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
    fn ref_register_struct<'a>(&'a self, types: &KTypes, proc: &'a Procedure, reg: Register) -> (KType, RefToUnknown<'a>) {
        match reg {
            Register::Arg(a) => { 
                let args = types.get_structure(proc.args);
                (args.fields[a].practical_type, self.args.ref_field(&args, a))
            }
            Register::Local(l) => { 
                let locals = types.get_structure(proc.locals);
                (locals.fields[l].practical_type, self.locals.ref_field(&locals, l))
            }
        }
    }

    fn mut_register2_struct<'a>(&'a mut self, types: &KTypes, proc: &'a Procedure, reg: Register, reg2: Register) -> (
        (KType, MutToUnknown<'a>), (KType, MutToUnknown<'a>),
    ) {
        assert_ne!(reg, reg2);
        let self2 = self as *mut Self;
        let (kt1, mtu1) = unsafe {&mut *self2}.mut_register_struct(types, proc, reg);
        let (kt2, mtu2) = unsafe {&mut *self2}.mut_register_struct(types, proc, reg2);
        ((kt1, mtu1), (kt2, mtu2))
    }

    fn mut_register_struct<'a>(&'a mut self, types: &KTypes, proc: &'a Procedure, reg: Register) -> (KType, MutToUnknown<'a>) {
        match reg {
            Register::Arg(a) => { 
                let args = types.get_structure(proc.args);
                (args.fields[a].practical_type, self.args.mut_field(&args, a))
            }
            Register::Local(l) => { 
                let locals = types.get_structure(proc.locals);
                (locals.fields[l].practical_type, self.locals.mut_field(&locals, l))
            }
        }
    }
}