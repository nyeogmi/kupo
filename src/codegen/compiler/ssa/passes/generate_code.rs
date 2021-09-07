use moogle::*;

use std::collections::HashMap;

use crate::codegen::GenInstruction;
use crate::codegen::compiler::ssa::SSA;
use crate::codegen::compiler::ssa::types::{ArgRegister, SSABlock};

use super::ControlFlowGraph;

use crate::codegen::code::Instruction as VMInstruction;

impl SSA {
    pub(in crate::codegen::compiler::ssa) 
    fn generate_code(self, cfg: &ControlFlowGraph) -> Vec<VMInstruction> {
        let mut block_addresses: HashMap<Id<SSABlock>, usize> = HashMap::new();
        let mut code = vec![];

        for (i, block) in cfg.approximate_toposort.iter().enumerate() {
            block_addresses.insert(*block, code.len());
            let data = self.blocks.get(*block).unwrap();

            for inst in &data.instructions {
                // -- don't write jumps that are unnecessary: ex, because they take us to the next block --
                // TODO: Handle various flavors of conditional jump
                if let GenInstruction::Jump { label: l2 } = inst {
                    if i + 1 < cfg.approximate_toposort.len() && *l2 == cfg.approximate_toposort[i + 1] {
                        continue
                    }
                } 

                code.push(*inst)
            }
        }

        let mut code2 = vec![];
        for instruction in code {
            code2.push(
                instruction
                .map_read(|a| match a {
                    ArgRegister::BParam(_) => panic!("a block param was left sitting around"),
                    ArgRegister::Known(k) => self.resolve_ssa_reg(k),
                })
                .map_write(|w| self.resolve_ssa_reg(w))
                .map_label(|l| block_addresses[&l])
            )
        }

        code2
    }
}