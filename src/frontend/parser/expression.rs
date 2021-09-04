use crate::frontend::lexer::{Grouping, Token};
use crate::frontend::located::Located;

use super::error_helpers::kpe;
use super::{Parse, Parser};

use super::internal_ast::*;

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self) -> Parse<ASTExpression> {
        // TODO: parse_expression_leaf and parse_expression_coda
        if let loc_t @ Located { value: Token::StringLiteral(s) , ..} = &self.ts.peek_any() {
            let loc = loc_t.location();
            let result = loc.replace(ASTExpression::StringLiteral { it: loc.replace(s.clone()) });
            self.ts.pop_any();
            result
        } else if self.ts.peek_identifier() {
            let call = self.parse_call();
            call.location().replace(ASTExpression::Call { call })
        } else {
            self.skip_to_end_of_expression().replace(
                ASTExpression::Invalid(kpe("expected expression"))
            )
        }
    }

    pub fn parse_assign_target(&mut self) -> Parse<ASTAssignTarget> {
        self.brack_comma_sep(
            |s| !s.ts.peek_tpred(|t| t == &Token::Grouping(Grouping::RBrack)),
            |s| s.parse_expression(),
            |args| ASTAssignTarget::Target { args },
            ASTAssignTarget::Invalid,
        )
    }

    pub fn parse_call(&mut self) -> Parse<ASTCall> {
        let name = if let Some(name) = self.ts.pop_identifier() { 
            name 
        } else {
            return self.give_up("expected function name for call", ASTCall::Invalid)
        };

        let args = self.parens(
            |s| ASTCallArgs::Args { args: 
                s.separated(
                    |s| !s.ts.peek_tpred(|t| t == &Token::Grouping(Grouping::RParen)),
                    |s| s.parse_expression(),
                    Grouping::Comma,
                ).value
            },
            ASTCallArgs::Invalid
        );

        name.location().merge_l(&args).replace(ASTCall::Call { name, args })
    }
}