use crate::frontend::lexer::Token;
use crate::frontend::located::Located;
use crate::frontend::{lexer::Grouping};

use super::grouping_helpers::DelimitedMany;
use super::internal_ast::*;

use super::{Parse, Parser, error_helpers::kpe};


impl<'a> Parser<'a> {
    pub fn parse_module(&mut self) -> Parse<ASTModule> {
        let mut delimit = DelimitedMany::braces_basis();
        delimit.separator = None;
        delimit.lhs = None;
        delimit.rhs = (Token::EOF, "end of file");

        self.group(
            delimit,
            |s| s.parse_item(),
            |items| ASTModule::Module { items },
            ASTModule::Invalid
        )
    }

    fn parse_item(&mut self) -> Parse<ASTItem> {
        if self.ts.peek_keyword("def") {
            let procedure = self.parse_def();
            procedure.locmap(ASTItem::Def)
        }
        else if self.ts.peek_keyword("view") {
            let procedure = self.parse_view();
            procedure.locmap(ASTItem::View)
        } 
        else {
            self.skip_to_next_item().replace(
                ASTItem::Invalid(kpe("unrecognized item: expected def or view"))
            )
        } 
    }

    fn parse_def(&mut self) -> Parse<ASTDef> {
        self.located(|s| {
            if s.ts.pop_keyword("def").is_none() {
                return ASTDef::Invalid(kpe("expected def"));
            };
            let name = if let Some(fn_name) = s.ts.pop_identifier() { fn_name } else {
                return ASTDef::Invalid(kpe("function name expected"));
            };
            let args = s.parse_args_parens();

            let return_type = if s.ts.peek_eq(&Token::Grouping(Grouping::LBrack)) {
                Some(s.parse_types_bracks())
            } else {
                None
            };

            let body = s.parse_block();

            ASTDef::Def { name, args, return_type, body }
        })
    }

    fn parse_view(&mut self) -> Parse<ASTView> {
        self.located(|s| {
            if s.ts.pop_keyword("view").is_none() {
                return ASTView::Invalid(kpe("expected view"));
            };
            let args = s.parse_args_bracks();

            if s.ts.pop_keyword("in").is_none() {
                return ASTView::Invalid(kpe("expected in"));
            };

            let name = if let Some(name) = s.ts.pop_identifier() { name } else {
                return ASTView::Invalid(kpe("view name expected"));
            };

            let body = s.parse_block_query_expression();
            let mut clauses = vec![body];

            while s.ts.pop_keyword("or").is_some() {
                clauses.push(s.parse_block_query_expression());
            }

            return ASTView::View { name, args, clauses }
        })
    }

    fn parse_args_parens(&mut self) -> Parse<ASTArgs> {
        let mut delimit = DelimitedMany::parens_basis();
        delimit.separator = Some(Token::Grouping(Grouping::Comma));
        self.group(
            delimit,
            |s| s.parse_arg(),
            |args| ASTArgs::Args { args },
            ASTArgs::Invalid
        )
    }

    fn parse_args_bracks(&mut self) -> Parse<ASTArgs> {
        let mut delimit = DelimitedMany::brackets_basis();
        delimit.can_be_bare = true;
        delimit.separator = Some(Token::Grouping(Grouping::Comma));
        self.group(
            delimit,
            |s| s.parse_arg(),
            |args| ASTArgs::Args { args },
            ASTArgs::Invalid
        )
    }

    fn parse_arg(&mut self) -> Located<ASTArg> {
        self.located(|s| {
            let name = if let Some(name) = s.ts.pop_variable() { name } else {
                return ASTArg::Invalid(kpe("expected string"));
            };
            let type_name = s.parse_optional_type();
            let loc = type_name.location();
            let type_name = match type_name.value {
                Some(x) => Some(loc.replace(x)),
                None => None,
            };
            ASTArg::Arg { name, type_name }
        })
    }

    fn parse_types_bracks(&mut self) -> Parse<ASTTypes> {
        let mut delimit = DelimitedMany::brackets_basis();
        delimit.separator = Some(Token::Grouping(Grouping::Comma));
        self.group(
            delimit,
            |s| s.parse_type(),
            |types| ASTTypes::Types { types },
            ASTTypes::Invalid
        )
    }

    fn parse_type(&mut self) -> Parse<ASTType> {
        self.located(|s| {
            if let Some(name) = s.ts.pop_identifier() {
                ASTType::Type { name }
            }
            else {
                ASTType::Invalid(kpe("expected type"))
            }
        })
    }

    fn parse_optional_type(&mut self) -> Parse<Option<ASTType>> {
        self.located(|s| {
            if let Some(name) = s.ts.pop_identifier() {
                Some(ASTType::Type { name })
            }
            else {
                None
            }
        })
    }
}