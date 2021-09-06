use moogle::*;

use crate::codegen::{GenInstruction, compiler::ssa::SSA};

use super::{BlockNeeds, ControlFlowGraph};

impl SSA {
    pub(in crate::codegen::compiler::ssa) 
    fn prepare_initial_block(&mut self, cfg: &ControlFlowGraph, needs: &BlockNeeds) {
        // -- Populate all variables for entry block --
        let entry_block = cfg.entry_block;
        let needs_fwd = needs.block_needs.fwd();
        for need in needs_fwd.iter() {
            // TODO: Read them from the prototype, or fail in a colorful way
            todo!();
        }

        self.add_instruction(entry_block, GenInstruction::Jump { label: cfg.first_real_block });
    }

    pub(in crate::codegen::compiler::ssa) 
    fn generate_final_block(&mut self, cfg: &ControlFlowGraph) {
        let final_block = self.create_block();
    
        for block in cfg.reachable_blocks.fwd().iter() {
            let data = self.blocks.get_mut(block).unwrap();

            // -- redirect existing blocks to jump to final block --
            let last = data.last_instruction_mut().unwrap();
            if let GenInstruction::Return{} = last {
                *last = GenInstruction::Jump{ label: final_block }
            }
        }

        // -- generate content of final block --
        // TODO: What variables need to be set before we return? Read them first.
        self.add_instruction(final_block, GenInstruction::Return {});
    }
}