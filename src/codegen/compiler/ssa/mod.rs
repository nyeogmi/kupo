use moogle::*;
use moogle::methods::*;

use self::types::*;

mod codegen_main;
mod passes;
mod types;

pub struct SSA {
    blocks: RawPom<SSABlock>,
    registers: RawPom<SSAReg>,
}

impl SSA {
    pub fn create_block(&mut self) -> Id<SSABlock> {
        self.blocks.insert(SSABlock {
            complete: false,
            parameters: vec![],
            instructions: vec![],
            populates: RawToOne::new(),
        })
    }

    pub fn create_register(&mut self) -> Id<SSAReg> {
        self.registers.insert(SSAReg)
    }

    pub fn read_variable(&mut self, block: Id<SSABlock>, variable: Id<Var>) -> ArgRegister {
        let data = self.blocks.get_mut(block).unwrap();
        if let Some(v) = data.populates.fwd().get(variable) {
            ArgRegister::Known(v)
        }
        else if let Some(ix) = data.parameters.iter().position(|&v| v == variable) {
            ArgRegister::BParam(ix)
        }
        else {
            let ix = data.parameters.len();
            data.parameters.push(variable.to_owned());
            ArgRegister::BParam(ix)
        }
    }

    pub fn write_variable(&mut self, block: Id<SSABlock>, variable: Id<Var>) -> Id<SSAReg> {
        let data = self.blocks.get_mut(block).unwrap();
        if data.complete {
            panic!("can't write variable in complete block");
        }

        let reg = self.create_register();
        self.write_variable_reg(block, variable, reg)
    }

    pub fn write_variable_reg(&mut self, block: Id<SSABlock>, variable: Id<Var>, register: Id<SSAReg>) -> Id<SSAReg> {
        let data = self.blocks.get_mut(block).unwrap();
        if data.complete {
            panic!("can't write variable in complete block");
        }

        data.populates.mut_fwd().insert(variable, register);
        register
    }

    pub fn add_instruction(&mut self, block: Id<SSABlock>, instruction: SSAInstruction) {
        let data = self.blocks.get_mut(block).unwrap();
        if data.complete {
            panic!("can't add instruction to complete block");
        }
        data.instructions.push(instruction);
        if instruction.is_jump() { data.complete = true; }
    }
}