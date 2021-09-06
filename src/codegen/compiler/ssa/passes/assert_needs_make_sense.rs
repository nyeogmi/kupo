use moogle::*;

use crate::codegen::compiler::ssa::SSA;

use super::{BlockNeeds, ControlFlowGraph};

impl SSA {
    pub(in crate::codegen::compiler::ssa) 
    fn assert_needs_make_sense(&self, cfg: &ControlFlowGraph, needs: &BlockNeeds) {
        // per lower(): we calculated these pre-phantoms, so there are no phantoms with phantom needs
        for target in &cfg.reachable_blocks {
            let needs_fwd = needs.block_needs.fwd();
            let needs = needs_fwd.get(target);
            if needs.len() == 0 { continue; }

            let jumpers_bwd = cfg.block_jumps_to.bwd();
            let jumpers = jumpers_bwd.get(target);

            let target_data = self.blocks.get(target).unwrap();

            if target == cfg.entry_block {
                // nothing is allowed to jump to the entry block
                assert_eq!(0, jumpers.len());

                // make sure the entry block populates everything
                for need in needs.iter() {
                    assert!(target_data.populates.fwd().contains_key(need));
                }

            } else if jumpers.len() > 1 {
                // make sure the phantom process worked
                assert_eq!(0, target_data.parameters.len());
                for need in needs.iter() {
                    assert!(target_data.populates.fwd().contains_key(need));
                }
            } else {
                // Populate it in the next procedure
            }
        }

        // TODO: Make sure the return point sets all the variables that need to be set
    }
}