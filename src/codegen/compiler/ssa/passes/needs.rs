use std::collections::VecDeque;

use moogle::*;

use crate::codegen::compiler::ssa::{SSA, types::{SSABlock, Var}};

use super::ControlFlowGraph;


pub(crate) struct BlockNeeds {
    pub block_needs: RawToMany<Id<SSABlock>, Id<Var>>
}

impl SSA {
    pub(in crate::codegen::compiler::ssa) 
    fn block_needs(&self, cfg: &ControlFlowGraph) -> BlockNeeds {
        // == Calculate total needs of all blocks ==
        let mut block_needs_queue = VecDeque::new();
        let mut block_needs = RawToMany::new();
        let mut need = |q: &mut VecDeque<_>, block, need| {
            if !block_needs.fwd().contains(block, need) {
                block_needs.mut_fwd().insert(block, need);
                q.push_back((block, need));
            }
        };
        for (block, data) in self.blocks.iter() {
            for p in data.parameters.iter() { need(&mut block_needs_queue, block, *p); }
        }

        while let Some((block, var)) = block_needs_queue.pop_front() {
            for jumper in cfg.block_jumps_to.bwd().get(block).iter() {
                let data = self.blocks.get(jumper).unwrap();
                if !data.populates.fwd().contains_key(var) {
                    need(&mut block_needs_queue, jumper, var)
                }
            }
        }

        BlockNeeds { block_needs }
    }
}