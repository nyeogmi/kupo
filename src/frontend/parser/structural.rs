use crate::frontend::located::Located;
use crate::frontend::{lexer::Grouping};

use super::internal_ast::*;

use super::{Parse, Parser, error_helpers::kpe};


impl<'a> Parser<'a> {
    pub fn parse_module(&mut self) -> Parse<ASTModule> {
        let items = self.many(
            |s| s.ts.any(), 
            |s| s.parse_item()
        );
        items.locmap(|items| ASTModule { items })
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
        let def = if let Some(def) = self.ts.pop_keyword("def") { def } else {
            return self.give_up("expected def", ASTDef::Invalid);
        };
        let name = if let Some(fn_name) = self.ts.pop_identifier() { fn_name } else {
            return def.merge_r(self.give_up(
                "function name expected",
                ASTDef::Invalid
            ));
        };
        let args = self.parse_args_parens();
        let body = self.parse_block();

        def.merge_l(&body).replace(ASTDef::Def {
            name,
            args,
            body
        })
    }

    fn parse_view(&mut self) -> Parse<ASTView> {
        let loc = self.ts.location();
        let _ = if let Some(view) = self.ts.pop_keyword("view") { view } else {
            return self.give_up("expected view", ASTView::Invalid);
        };
        let args = self.parse_args_bracks();

        let _ = if let Some(in_) = self.ts.pop_keyword("in") { in_ } else {
            return self.give_up("expected in", ASTView::Invalid)
        };

        let name = if let Some(name) = self.ts.pop_identifier() { name } else {
            return self.give_up(
                "view name expected",
                ASTView::Invalid
            );
        };

        let body = self.parse_block_query_expression();
        let mut clauses = vec![body];

        while let Some(or_) = self.ts.pop_keyword("or") {
            let c = or_.merge_r(self.parse_block_query_expression());
            loc.merge_l(&c);
            clauses.push(c);
        }

        return loc.replace(ASTView::View { name, args, clauses })
    }

    fn parse_args_parens(&mut self) -> Parse<ASTArgs> {
        self.parens(
            |s| {
                ASTArgs::Args { args: s.separated(
                    |s,| s.peek_arg(),
                    |s,| s.parse_arg(),
                    Grouping::Comma,
                ).value}
            },
            ASTArgs::Invalid
        )
    }

    fn parse_args_bracks(&mut self) -> Parse<ASTArgs> {
        self.brack_comma_sep(
            |s| s.peek_arg(),
            |s| s.parse_arg(),
            |args| ASTArgs::Args { args },
            ASTArgs::Invalid
        )
    }

    fn peek_arg(&mut self) -> bool { self.ts.peek_identifier() }
    fn parse_arg(&mut self) -> Located<ASTArg> {
        let loc = self.ts.location();
        let name = if let Some(name) = self.ts.pop_variable() { name } else {
            return self.give_up("expected string", ASTArg::Invalid);
        };
        let type_name = self.ts.pop_identifier();
        match type_name {
            Some(tname) => {
                loc.merge_l(&name).merge_l(&tname).replace(ASTArg::Arg { name, type_name: Some(tname) })
            }
            None => {
                loc.merge_l(&name).replace(ASTArg::Arg { name, type_name: None })
            }
        }
    }
}