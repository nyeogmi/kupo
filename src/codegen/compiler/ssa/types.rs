use std::{collections::{HashMap, VecDeque}};

use moogle::*;
use moogle::methods::*;

use crate::codegen::GenInstruction;
use crate::codegen::Instruction as VMInstruction;

pub type SSAInstruction = GenInstruction<Id<SSAReg>, ArgRegister, Id<SSABlock>>;

pub struct Var;
pub struct SSAReg;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArgRegister { BParam(usize), Known(Id<SSAReg>) }

pub struct SSABlock {
    pub(super) complete: bool,
    pub(super) parameters: Vec<Id<Var>>,
    pub(super) instructions: Vec<SSAInstruction>,
    pub(super) populates: RawToOne<Id<Var>, Id<SSAReg>>,
}

impl SSABlock {
    pub fn last_instruction_mut(&mut self) -> Option<&mut SSAInstruction> {
        let len = self.instructions.len();
        if len == 0 { None }
        else { Some(&mut self.instructions[len - 1]) }
    }

    pub fn pipe_in_variable(&mut self, var: Id<Var>, register: Id<SSAReg>) {
        if let Some(param_index) = self.parameters.iter().position(|v| v == &var) {
            self.parameters.remove(param_index);
            for i in &mut self.instructions {
                *i = i.replace_read(&ArgRegister::BParam(param_index), || ArgRegister::Known(register))
            }
        } 
        // if nothing else is there, make sure this variable is!
        if self.populates.fwd().get(var).is_some() {
            self.populates.mut_fwd().insert(var, register);
        }
    }
}