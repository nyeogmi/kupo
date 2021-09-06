use crate::frontend::ast;
use crate::frontend::Located;

use super::StructBuilder;

fn compile_view(view: Located<ast::View>) {
    let (loc, it) = view.split();

    for clause in it.clauses {
        compile_view_clause(clause)
    }
}

fn compile_view_clause(view_clause: Located<ast::QueryExpression>) {
    let (loc, it) = view_clause.split();

    for goal in it.items {
        compile_goal(goal)
    }
}

fn compile_goal(goal: Located<ast::QueryGoal>) {
    let (loc, it) = goal.split();

    match it {
        QueryGoal::In { args, from } => {
            todo!();
        }
        QueryGoal::Assign { args, expression } => {
            todo!();
        }
    }
}