use crate::frontend::lexer::{Grouping, Operator, Token};

use super::error_helpers::kpe;
use super::{Parse, Parser};

use super::internal_ast::*;

impl<'a> Parser<'a> {
    pub fn parse_block_query_expression(&mut self) -> Parse<ASTQueryExpression> {
        // TODO: Check for brackets, then parse query expression
        self.braces(|s| s.parse_query_expression().value, ASTQueryExpression::Invalid)
    }

    pub fn parse_query_expression(&mut self) -> Parse<ASTQueryExpression> {
        let items = self.separated(
            |s| !s.ts.peek_tpred(|t| t == &Token::Grouping(Grouping::RBrace)), 
            |s| { s.parse_query_goal() }, 
            Grouping::Comma,
        );
        items.locmap(|items| ASTQueryExpression::QExpression { items })
    }

    fn parse_query_goal(&mut self) -> Parse<ASTQueryGoal> {
        let args = self.parse_assign_target();
        let source = self.parse_goal_source();

        args.location().merge_l(&source).replace(ASTQueryGoal::Goal { args, source })
    }

    fn parse_goal_source(&mut self) -> Parse<ASTQueryGoalSource> {
        if self.ts.peek_keyword("in") {
            let in_ = self.ts.pop_keyword("in").unwrap();
            let tbl = if let Some(ident) = self.ts.pop_identifier() { ident } else {
                return self.give_up("expected table after 'in'", ASTQueryGoalSource::Invalid);
            };
            in_.location().merge_l(&tbl).replace(ASTQueryGoalSource::In { from: tbl })

        } else if self.ts.peek_tpred(|t| t == &Token::Operator(Operator::OAssignNew)) {
            let assign = self.ts.pop_any(); 
            let expression = self.parse_expression();
            assign.location().merge_l(&expression).replace(ASTQueryGoalSource::Assign { expression })

        } else if self.ts.peek_tpred(|t| t == &Token::Operator(Operator::OAssign)) {
            let assign = self.ts.pop_any(); 
            let expression = self.parse_expression();
            assign.location().merge_l(&expression).replace(ASTQueryGoalSource::Invalid(kpe(
                "you can't use = in a query expression, only :="
            )))

        } else {
            let garbage = self.ts.pop_any(); 
            return garbage.replace(
                ASTQueryGoalSource::Invalid(kpe("unrecognized goal source: expected in or :="))
            )
        }
    }
}