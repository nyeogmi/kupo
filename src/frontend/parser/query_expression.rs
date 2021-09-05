use crate::frontend::lexer::{Grouping, Operator, Token};

use super::error_helpers::kpe;
use super::grouping_helpers::DelimitedMany;
use super::{Parse, Parser};

use super::internal_ast::*;

impl<'a> Parser<'a> {
    pub fn parse_block_query_expression(&mut self) -> Parse<ASTQueryExpression> {
        let mut delimit = DelimitedMany::braces_basis();
        delimit.separator = Some(Token::Grouping(Grouping::Comma));

        self.group(
            delimit,
            |s| s.parse_query_goal(),
            |items| ASTQueryExpression::QExpression { items },
            ASTQueryExpression::Invalid,
        )
    }

    pub fn parse_plain_query_expression(&mut self, rhs: (Token, &'static str)) -> Parse<ASTQueryExpression> {
        let mut delimit = DelimitedMany::braces_basis();
        delimit.separator = Some(Token::Grouping(Grouping::Comma));

        delimit.lhs = None;
        delimit.rhs = rhs;
        delimit.consume_rhs = false;

        self.group(
            delimit,
            |s| s.parse_query_goal(),
            |items| ASTQueryExpression::QExpression { items },
            ASTQueryExpression::Invalid,
        )
    }

    fn parse_query_goal(&mut self) -> Parse<ASTQueryGoal> {
        let args = self.parse_assign_target();
        let source = self.parse_goal_source();

        args.location().merge_l(&source).replace(ASTQueryGoal::Goal { args, source })
    }

    fn parse_goal_source(&mut self) -> Parse<ASTQueryGoalSource> {
        return self.located(|s| {
            if s.ts.pop_keyword("in").is_some() {
                let tbl = if let Some(ident) = s.ts.pop_identifier() { ident } else {
                    return ASTQueryGoalSource::Invalid(kpe("expected table after 'in'"))
                };
                ASTQueryGoalSource::In { from: tbl }

            } else if s.ts.pop_eq(&Token::Operator(Operator::OAssignNew)).is_some() {
                let expression = s.parse_expression();
                ASTQueryGoalSource::Assign { expression }

            } else if s.ts.pop_eq(&Token::Operator(Operator::OAssign)).is_some() {
                s.parse_expression();
                ASTQueryGoalSource::Invalid(kpe(
                    "you can't use = in a query expression, only :="
                ))

            } else {
                s.ts.pop_any(); 
                // TODO: Attempt to move all the way onto RHS delimiter
                ASTQueryGoalSource::Invalid(kpe("unrecognized goal source: expected in or :="))
            }
        })
    }
}