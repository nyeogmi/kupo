use crate::frontend::lexer::{Grouping, Operator, Token};

use super::error_helpers::kpe;
use super::{Parse, Parser};

use super::internal_ast::*;

impl<'a> Parser<'a> {
    pub fn parse_block(&mut self) -> Parse<ASTBlock> {
        self.braces(|s| 
            ASTBlock::Block { items: s.separated(
                |s| s.ts.peek_tpred(|t| t != &Token::Grouping(Grouping::RBrace)),
                |s| s.parse_statement(),
                Grouping::Semicolon
            ).value},
            ASTBlock::Invalid
        )
    }

    pub fn parse_statement(&mut self) -> Parse<ASTStatement> {
        if let Some(for_) = self.ts.pop_keyword("for") {
            let arg = self.parse_query_expression();
            let body = self.parse_block();
            for_.location().merge_l(&body).replace(ASTStatement::For { arg, body })
        }
        else if let Some(if_) = self.ts.pop_keyword("if") {
            let arg = self.parse_query_expression();
            let body = self.parse_block();
            if let Some(_)  = self.ts.pop_keyword("else") {
                let else_ = self.parse_block();
                if_.location().merge_l(&else_).replace(ASTStatement::If { arg, body, else_: Some(else_)})
            } else {
                if_.location().merge_l(&body).replace(ASTStatement::If { arg, body, else_: None })
            }
        } else if let Some(return_) = self.ts.pop_keyword("return") {
            let arg = self.parse_expression();
            return_.location().merge_l(&arg).replace(ASTStatement::Return { arg })
        } else if let Some(identifier) = self.ts.pop_identifier() {
            let call = self.parse_call();
            let _ = call.location();
            identifier.location().merge_l(&call).replace(ASTStatement::Call { call })

        // TODO: check for [ or a variable to make sure it's likely this was even intended as an assign target
        // we can do this because at this point if it's not met, we have no clue what's goign on
        } else if self.ts.peek_variable() || self.ts.peek_tpred(|t| t == &Token::Grouping(Grouping::LBrack)) {
            let variable = self.parse_assign_target();
            let first = 
                if let Some(_) = self.ts.pop_tpred(|t| t == &Token::Operator(Operator::OAssign)) {
                    false
                } else if let Some(_) = self.ts.pop_tpred(|t| t == &Token::Operator(Operator::OAssignNew)) { 
                    true
                } else {
                    return self.give_up("expected := or = for an assignment", ASTStatement::Invalid)
                };
            let arg = self.parse_expression();
            variable.location().merge_l(&arg).replace(ASTStatement::Assign { first, variable, arg })
        } else {
            // TODO: Skip to the next likely statement start
            self.skip_to_next_statement().replace(
                ASTStatement::Invalid(kpe("expected statement"))
            )
        }
    }
} 