use moogle::*;

use std::collections::VecDeque;

use crate::codegen::compiler::ssa::SSA;

use super::{BlockNeeds, ControlFlowGraph};


impl SSA {
    pub(in crate::codegen::compiler::ssa) 
    fn eliminate_block_params(&mut self, cfg: &ControlFlowGraph, needs: &BlockNeeds) {
        // propagate needs downwards
        let mut block_has_queue = VecDeque::new();
        let mut block_has = RawToMany::new();
        let mut has = |q: &mut VecDeque<_>, block, need, reg| {
            if block_has.fwd().contains(block, need) {
                panic!("reached the same block by jumping from two places while propagating needs; phantom elimination is supposed to avoid this")
            }
            block_has.mut_fwd().insert(block, need);
            q.push_back((block, need, reg));
        };

        for (var, reg) in self.blocks.get(cfg.entry_block).unwrap().populates.fwd().iter() {
            block_has_queue.push_back((cfg.entry_block, var, reg))
        }

        while let Some((jumper, var, jumper_reg)) = block_has_queue.pop_front() {
            for target in cfg.block_jumps_to.fwd().get(jumper).iter() {
                let data = self.blocks.get_mut(target).unwrap();
                data.pipe_in_variable(var, jumper_reg);
                has(&mut block_has_queue, target, var, data.populates.fwd().get(var).unwrap())
            }
        }
    }
}