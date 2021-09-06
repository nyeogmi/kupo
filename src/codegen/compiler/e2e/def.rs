use crate::{codegen::{Program, compiler::ssa::SSA}, frontend::{Located, ast}};

pub fn create_def_prototype(program: &Program, def: Located<&ast::Def>) {

}

pub fn compile_def(program: &mut Program, def: Located<ast::Def>) {
    let ssa = SSA::new();
    todo!();
}
