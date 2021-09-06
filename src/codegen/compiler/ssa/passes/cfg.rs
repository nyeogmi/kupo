use std::collections::VecDeque;

use moogle::*;

use crate::codegen::compiler::ssa::SSA;
use crate::codegen::compiler::ssa::types::SSABlock;

pub(in crate::codegen::compiler::ssa) 
struct ControlFlowGraph {
    pub entry_block: Id<SSABlock>,
    pub first_real_block: Id<SSABlock>,
    pub reachable_blocks: RawSet<Id<SSABlock>>,
    pub block_jumps_to: RawManyToMany<Id<SSABlock>, Id<SSABlock>>,
    pub approximate_toposort: Vec<Id<SSABlock>>,  // toposort with arbitrary order in cycles
}

impl SSA {
    pub(in crate::codegen::compiler::ssa) 
    fn analyze_cfg(&self) -> ControlFlowGraph {
        // == Make sure there's an entry block ==
        let entry_block = self.blocks.keys().nth(0).unwrap();

        // == Make sure there's a user block ==
        let first_real_block = self.blocks.keys().nth(1).unwrap();

        // == Make sure all blocks are complete ==
        for (_, data) in self.blocks.iter() {
            if !data.complete { panic!("can't lower blocks: some are incomplete"); }
        }

        // == Calculate block jump table ==
        let mut block_jumps_to = RawManyToMany::new();
        for (block, data) in self.blocks.iter() {
            data.instructions[data.instructions.len() - 1].for_jumped_labels(|block2| {
                block_jumps_to.mut_fwd().insert(block, *block2);
            })
        }

        // == Make sure all blocks are reachable ==
        let mut reachable_blocks = RawSet::new();
        let mut reachable_blocks_queue = VecDeque::new();
        let mut approximate_toposort = vec![];

        reachable_blocks_queue.push_back(entry_block);
        let mut sweep = |q: &mut VecDeque<_>, block| {
            if !reachable_blocks.fwd().contains(block) {
                reachable_blocks.mut_fwd().insert(block);
                q.push_back(block);
            }
        };
        while let Some(block) = reachable_blocks_queue.pop_front() {
            approximate_toposort.push(block);
            for target in block_jumps_to.fwd().get(block).iter() {
                sweep(&mut reachable_blocks_queue, target)
            }
        }
        
        ControlFlowGraph {
            entry_block,
            first_real_block,
            reachable_blocks,
            block_jumps_to,
            approximate_toposort,
        }
    }
}