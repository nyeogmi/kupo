use std::{collections::{HashMap, VecDeque}};

use moogle::*;
use crate::codegen::Instruction as VMInstruction;

use super::{SSA, types::SSAReg};

impl SSA {
    pub fn lower(mut self) -> Vec<VMInstruction> {
        // Generate final block
        let cfg = self.analyze_cfg();
        self.generate_final_block(&cfg);

        let cfg = self.analyze_cfg();
        let needs = self.block_needs(&cfg);
        self.generate_phantoms(&cfg, &needs);
        let cfg = self.analyze_cfg();

        self.prepare_initial_block(&cfg, &needs);

        // We're about to populate all the non-entry blocks!!! 
        // Make sure we didn't screw anything up earlier
        self.assert_needs_make_sense(&cfg, &needs);
        self.eliminate_block_params(&cfg, &needs);

        // self.infer_types();
        // self.assign_physical_registers();

        self.generate_code(&cfg)
    }

    pub(crate) fn resolve_ssa_reg(&self, w: Id<SSAReg>) -> crate::codegen::Register {
        todo!()
    }
}