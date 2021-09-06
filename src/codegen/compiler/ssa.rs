use std::{collections::{HashMap, VecDeque}};

use moogle::*;
use moogle::methods::*;

use crate::codegen::GenInstruction;
use crate::codegen::Instruction as VMInstruction;

type SSAInstruction = GenInstruction<Id<SSAReg>, ArgRegister, Id<SSABlock>>;

pub struct SSA {
    blocks: RawPom<SSABlock>,
    registers: RawPom<SSAReg>,
}

pub struct SSABlock {
    complete: bool,
    parameters: Vec<Id<Var>>,
    instructions: Vec<SSAInstruction>,
    populates: RawToOne<Id<Var>, Id<SSAReg>>,
}

impl SSABlock {
    fn last_instruction_mut(&mut self) -> Option<&mut SSAInstruction> {
        let len = self.instructions.len();
        if len == 0 { None }
        else { Some(&mut self.instructions[len - 1]) }
    }

    fn pipe_in_variable(&mut self, var: Id<Var>, register: Id<SSAReg>) {
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

/*
struct SealedBlock {
    instructions: Vec<GenSSAInstruction<Id<SSAReg>>>,
    populates: HashMap<Id<Var>, Id<SSAReg>>,
}
*/

pub struct Var;

pub struct SSAReg;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ArgRegister { BParam(usize), Known(Id<SSAReg>) }


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

    fn generate_phantoms(&mut self, cfg: &ControlFlowGraph, needs: &BlockNeeds) {
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

    fn propagate_needs_downwards(&mut self, cfg: &ControlFlowGraph, needs: &BlockNeeds) {
        let mut block_has_queue = VecDeque::new();
        let mut block_has = RawToMany::new();
        let mut has = |q: &mut VecDeque<_>, block, need, reg| {
            if block_has.fwd().contains(block, need) {
                panic!("reached the same block by jumping from two places while propagating needs")
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

    fn generate_code(mut self, cfg: &ControlFlowGraph) -> Vec<VMInstruction> {
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
        self.propagate_needs_downwards(&cfg, &needs);

        self.generate_code(&cfg)
    }

    pub(crate) fn resolve_ssa_reg(&self, w: Id<SSAReg>) -> crate::codegen::Register {
        todo!()
    }
}

struct ControlFlowGraph {
    entry_block: Id<SSABlock>,
    first_real_block: Id<SSABlock>,
    reachable_blocks: RawSet<Id<SSABlock>>,
    block_jumps_to: RawManyToMany<Id<SSABlock>, Id<SSABlock>>,
    approximate_toposort: Vec<Id<SSABlock>>,  // toposort with arbitrary order in cycles
}

struct BlockNeeds {
    block_needs: RawToMany<Id<SSABlock>, Id<Var>>
}