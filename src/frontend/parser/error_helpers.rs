use crate::frontend::{lexer::{Grouping, Token}, located::Located};

use super::{Parse, Parser, internal_ast::KupoParseError};


// NYEO NOTE: Some of these are quite complicated!!!
// They're here to improve the error messages and are not a real part of the grammar.
impl<'a> Parser<'a> {
    // TODO: Replace with "skip_to_probable_next_item"
    pub fn skip_to_next_item(&mut self) -> Located<()> {
        let mut n_brace: isize = 0;
        let mut found_one: bool = false;
        let mut loc: Located<()> = self.ts.location();

        loop {
            if self.ts.peek_keyword("fn") { return loc; }
            if self.ts.peek_keyword("view") { return loc; }

            let t = self.ts.pop_any();
            loc = loc.merge_l(&t).location();

            if &t.value == &Token::Grouping(Grouping::LBrace) { 
                n_brace += 1; 
                found_one = true;
            }
            if &t.value == &Token::Grouping(Grouping::RBrace) { 
                n_brace -= 1; 
            }

            if t.value == Token::EOF || (n_brace <= 0 && found_one) {
                return loc
            }
        }
    }

    pub fn skip_to_next_statement(&mut self) -> Located<()> {
        // TODO:
        self.ts.pop_any().replace(())
    }

    pub fn skip_to_end_of_expression(&mut self) -> Located<()> {
        // TODO:
        self.ts.pop_any().replace(())
    }

    pub fn give_up<T>(&self, s: &str, f: impl FnOnce(KupoParseError) -> T) -> Parse<T> {
        return self.ts.location().replace(f(kpe(s)))
    }
}

pub fn kpe(s: &str) -> KupoParseError {
    KupoParseError(s.to_string())
}