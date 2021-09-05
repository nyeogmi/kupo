use crate::frontend::lexer::{Grouping, Operator, Token};

use super::error_helpers::kpe;
use super::grouping_helpers::DelimitedMany;
use super::{Parse, Parser};

use super::internal_ast::*;

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self) -> Parse<ASTExpression> {
        let mut leaf = self.parse_leaf_expression();

        loop {
            if self.ts.pop_eq(&Token::Operator(Operator::OAdd)).is_some() {
                let leaf2 = self.parse_leaf_expression();
                leaf = leaf.add_using_precedence(ASTBinOp::Add, leaf2);
            }
            else if self.ts.pop_eq(&Token::Operator(Operator::OSubtract)).is_some() {
                let leaf2 = self.parse_leaf_expression();
                leaf = leaf.add_using_precedence(ASTBinOp::Subtract, leaf2);
            }
            else if self.ts.pop_eq(&Token::Operator(Operator::OMultiply)).is_some() {
                let leaf2 = self.parse_leaf_expression();
                leaf = leaf.add_using_precedence(ASTBinOp::Multiply, leaf2);
            }
            else if self.ts.pop_eq(&Token::Operator(Operator::ODivide)).is_some() {
                let leaf2 = self.parse_leaf_expression();
                leaf = leaf.add_using_precedence(ASTBinOp::Divide, leaf2);
            }
            else {
                return leaf;
            }
        }
    }

    pub fn parse_leaf_expression(&mut self) -> Parse<ASTExpression> {
        self.located(|s| {
            // TODO: parse_expression_leaf and parse_expression_coda
            if s.ts.pop_eq(&Token::Operator(Operator::OSubtract)).is_some() {
                ASTExpression::UOp {
                    op: ASTUOp::Negate,
                    arg: Box::new(s.parse_leaf_expression()),
                }
            } else if s.ts.pop_eq(&Token::Operator(Operator::OAdd)).is_some() {
                ASTExpression::UOp {
                    op: ASTUOp::Plus,
                    arg: Box::new(s.parse_leaf_expression()),
                }
            } else if let Token::Integer(i) = &s.ts.peek_any().value {
                let result = ASTExpression::IntegerLiteral { it: i.clone() };
                s.ts.pop_any();
                result
            } else if let Token::StringLiteral(string) = &s.ts.peek_any().value {
                let result = ASTExpression::StringLiteral { it: string.clone() };
                s.ts.pop_any();
                result
            } else if s.ts.peek_identifier() {
                let call = s.parse_call();
                ASTExpression::Call { call }
            } else {
                s.skip_to_end_of_expression();
                ASTExpression::Invalid(kpe("expected expression"))
            }
        })
    }

    pub fn parse_assign_target(&mut self) -> Parse<ASTAssignTarget> {
        let mut delimit = DelimitedMany::brackets_basis();
        delimit.can_be_bare = true;
        delimit.separator = Some(Token::Grouping(Grouping::Comma));

        self.group(
            delimit,
            |s| s.parse_expression(),
            |args| ASTAssignTarget::Target { args },
            ASTAssignTarget::Invalid,
        )
    }

    pub fn parse_call(&mut self) -> Parse<ASTCall> {
        self.located(|s| {
            let name = if let Some(name) = s.ts.pop_identifier() { 
                name 
            } else {
                return ASTCall::Invalid(kpe("expected function name for call"))
            };

            let mut delimit = DelimitedMany::parens_basis();
            delimit.separator = Some(Token::Grouping(Grouping::Comma));

            let args = s.group(
                delimit,
                |s| s.parse_expression(),
                |args| ASTCallArgs::Args { args },
                ASTCallArgs::Invalid
            );

            ASTCall::Call { name, args }
        })
    }
}