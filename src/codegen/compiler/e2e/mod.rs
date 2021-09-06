mod def;
mod view;

use crate::codegen::Program;

use crate::frontend::{Located, ast};

use self::def::*;
use self::view::*;

pub fn compile(source_code: Located<ast::Module>) -> Program {
    let mut program = Program::new();

    for item in source_code.value.items.iter() {
        let (loc, it) = item.split();
        match it {
            ast::Item::Def(def) => { create_def_prototype(&program, loc.replace(&def)); }
            ast::Item::View(view) => { create_view_prototype(&program, loc.replace(&view)); }
        }
    }

    for item in source_code.value.items {
        let (loc, it) = item.split();
        match it {
            ast::Item::Def(def) => compile_def(&mut program, loc.replace(def)),
            ast::Item::View(view) => compile_view(&mut program, loc.replace(view)),
        }
    }

    program
}

