mod error_helpers;
mod expression;
mod grouping_helpers;
mod internal_ast;
mod statement;
mod structural;
mod query_expression;
mod tstream;

use tstream::*;

use self::internal_ast::*;

use super::{lexer::{Token}, located::Located};

struct Parser<'a> {
    ts: TStream<'a>,
}

type Parse<T> = Located<T>;

pub fn parse_module(ts: &[Located<Token>], eof: Located<()>) -> Parse<ASTModule> {
    let ts = TStream::new(ts, eof);
    Parser { ts }.parse_module()
}

impl<'a> Parser<'a> {
}