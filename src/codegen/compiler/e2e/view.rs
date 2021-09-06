use crate::{codegen::{Program, compiler::ssa::SSA}, frontend::{Located, ast}};

pub fn create_view_prototype(program: &Program, def: Located<&ast::View>) {

}

pub fn compile_view(program: &mut Program, view: Located<ast::View>) {
    let ssa = SSA::new();
    todo!();
}