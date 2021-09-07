use std::collections::HashMap;

use moogle::*;
use moogle::methods::*;

use disjoint_sets::UnionFind;

use crate::codegen::{compiler::ssa::{SSA, types::{ArgRegister, SSAReg}}, structures::KType};

impl SSA {
    pub(in crate::codegen::compiler::ssa) 
    fn infer_types(&mut self) {
        // -- Figure out the _full_ list of registers with the same type as each other --
        let mut same_type = UnionFind::new(self.registers.len());

        for i in self.registers.keys() {
            let element: usize = same_type.alloc();
            // this is true for current versions of disjoint_sets and we have no reason to upgrade
            assert_eq!(element, u(i))
        }

        // -- First: all registers associated with the same variable have the same type --
        for (_, set) in self.variable_registers.fwd().sets() {
            let canonical = set.iter().nth(0).unwrap();  // nonempty or it wouldn't have been returned by sets
            for reg in set.iter() {
                same_type.union(u(canonical), u(reg));
            }
        }

        // -- Next: look for copies. Registers on each side of a copy have the same type -- 
        for data in self.blocks.values() {
            for instruction in &data.instructions {
                instruction.for_copy(|w, r| {
                    match r {
                        &ArgRegister::Known(r) => { same_type.union(u(*w), u(r)); }
                        ArgRegister::BParam(_) => 
                            panic!("all block params should have been eliminated at this point")
                    }
                })
            }
        }

        // -- Now start figuring out types --
        let type_assertions: HashMap<usize, TypeAssertions> = HashMap::new();

        // -- Next: copy instruction-specific type asserts --
        /*
        for data in self.blocks.values() {
            for instruction in &data.instructions {
                instruction.for_typeasserts_read(|r| {
                    match r {
                        &ArgRegister::Known(r) => { todo!() }
                        ArgRegister::BParam(_) => 
                            panic!("all block params should have been eliminated at this point")
                    }
                }
                instruction.for_typeasserts_write(|w| {
                    todo!();
                }
            }
        }
        */

        // -- Next: use user's type assertions --
    }
}

struct TypeAssertions {
    assertions: Vec<KType>
}

// will work on all platforms for any function with less than 2**32-1 registers
fn u(id: Id<SSAReg>) -> usize {
    id.get_value() as usize
}