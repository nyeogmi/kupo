pub mod ast;
mod keywords;
mod located;
mod lexer;
mod parser;

pub use self::located::Located;
use self::parser::internal_ast::KupoParseError;

pub fn parse_module(s: &str) -> Result<Located<ast::Module>, Vec<Located<KupoParseError>>> {
    let (ts, eof) = lexer::lex(s);
    let internal_parse = parser::parse_module(&ts, eof);
    ast::simplify_module(internal_parse)
}