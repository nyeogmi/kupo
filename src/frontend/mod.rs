mod keywords;
mod located;
mod lexer;
mod parser;

pub use parser::*;

use self::located::Located;

pub fn parse_module(s: &str) -> Located<ASTModule> {
    let (ts, eof) = lexer::lex(s);
    parser::parse_module(&ts, eof)
}