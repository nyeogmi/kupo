use crate::frontend::lexer::{Grouping, Operator, Token};

use super::error_helpers::kpe;
use super::grouping_helpers::DelimitedMany;
use super::{Parse, Parser};

use super::internal_ast::*;

impl<'a> Parser<'a> {
    pub fn parse_block(&mut self) -> Parse<ASTBlock> {
        let mut delimit = DelimitedMany::braces_basis();
        delimit.separator = Some(Token::Grouping(Grouping::Semicolon));  // TODO: No semicolons?

        self.group(
            delimit,
            |s| s.parse_statement(),
            |items| ASTBlock::Block { items },
            ASTBlock::Invalid
        )
    }

    pub fn parse_statement(&mut self) -> Parse<ASTStatement> {
        self.located(|s| {
            if s.ts.pop_keyword("for").is_some() {
                let arg = s.parse_plain_query_expression(
                    (Token::Grouping(Grouping::LBrace), "start of block")
                );
                let body = s.parse_block();
                ASTStatement::For { arg, body }
            }
            else if s.ts.pop_keyword("if").is_some() {
                let arg = s.parse_plain_query_expression(
                    (Token::Grouping(Grouping::LBrace), "start of block")
                );
                let body = s.parse_block();
                if s.ts.pop_keyword("else").is_some() {
                    let else_ = s.parse_block();
                    ASTStatement::If { arg, body, else_: Some(else_)}
                } else {
                    ASTStatement::If { arg, body, else_: None }
                }
            } else if s.ts.pop_keyword("return").is_some() {
                let arg = s.parse_expression();
               ASTStatement::Return { arg }
            } else if s.ts.peek_identifier() {
                let call = s.parse_call();
                ASTStatement::Call { call }

            // TODO: check for [ or a variable to make sure it's likely this was even intended as an assign target
            // we can do this because at this point if it's not met, we have no clue what's goign on
            } else if s.ts.peek_variable() || s.ts.peek_eq(&Token::Grouping(Grouping::LBrack)) {
                let variable = s.parse_assign_target();
                let first = 
                    if s.ts.pop_eq(&Token::Operator(Operator::OAssign)).is_some() {
                        false
                    } else if s.ts.pop_eq(&Token::Operator(Operator::OAssignNew)).is_some() { 
                        true
                    } else {
                        return ASTStatement::Invalid(kpe("expected := or = for an assignment"))
                    };
                let arg = s.parse_expression();
                ASTStatement::Assign { first, variable, arg }
            } else {
                // TODO: Skip to the next likely statement start
                s.skip_to_next_statement();
                ASTStatement::Invalid(kpe("expected statement"))
            }
        })
    }
} 