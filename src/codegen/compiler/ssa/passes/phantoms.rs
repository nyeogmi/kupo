use moogle::methods::*;

use std::collections::HashMap;

use crate::codegen::{GenInstruction, compiler::ssa::SSA};

use super::{BlockNeeds, ControlFlowGraph};

impl SSA {
    pub(in crate::codegen::compiler::ssa) fn generate_phantoms(&mut self, cfg: &ControlFlowGraph, needs: &BlockNeeds) {
        // == Generate phantom blocks if needed ==
        // (Sometimes we don't actually need a phantom block because the variables propagated down are identical.)
        // (In that case, uh, we'll take them out later. That might be a TODO. We'll see!)
        for target in &cfg.reachable_blocks {
            let jumps_bwd = cfg.block_jumps_to.bwd();
            let needs_fwd = needs.block_needs.fwd();
            let jumpers = jumps_bwd.get(target);
            let target_needs = needs_fwd.get(target);
            
            if jumpers.len() > 1 && target_needs.len() > 0 {
                // -- generate sites for needs --
                let mut need_registers = HashMap::new();
                for need in target_needs.iter() {
                    let need_register = self.create_register();
                    need_registers.insert(need, need_register);
                }

                for jumper in jumpers.iter() {
                    // -- generate phantom --
                    let phantom = self.create_block();

                    // -- add needs to phantom --
                    for need in target_needs.iter() {
                        let read = self.read_variable(phantom, need);
                        let write = need_registers[&need];
                        self.write_variable_reg(phantom, need, write);
                        self.add_instruction(phantom, GenInstruction::Move { out: write, arg: read });
                    }

                    // -- make substitutions in target in advance --
                    {
                        let target_data = self.blocks.get_mut(target).unwrap();
                        for need in target_needs.iter() {
                            let write = need_registers[&need];
                            target_data.pipe_in_variable(need, write);
                        }
                    }

                    // -- make phantom jump to old target --
                    self.add_instruction(phantom, GenInstruction::Jump { label: target });

                    // -- patch jumper to jump to phantom --
                    let jumper_data = self.blocks.get_mut(jumper).unwrap();
                    let inst = jumper_data.last_instruction_mut().unwrap();
                    *inst = inst.replace_jump(&target, || phantom)
                }
            }
        }
    }
}
