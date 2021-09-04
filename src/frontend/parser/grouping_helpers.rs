use crate::frontend::{lexer::{Grouping, Token}, located::Located};

use super::{Parse, Parser, internal_ast::KupoParseError};

impl<'a> Parser<'a> {
    pub fn parens<T>(
        &mut self, 
        f: impl Fn(&mut Self) -> T,
        fail: impl Fn(KupoParseError) -> T,
    ) -> Parse<T> {
        self.grouped(
            f, fail,
            Grouping::LParen, Grouping::RParen,
            "left paren expected", "right paren expected"
        )
    }

    pub fn brackets<T>(
        &mut self, 
        f: impl Fn(&mut Self) -> T,
        fail: impl Fn(KupoParseError) -> T,
    ) -> Parse<T> {
        self.grouped(
            f, fail,
            Grouping::LBrack, Grouping::RBrack, 
            "left bracket expected", "right bracket expected"
        )
    }

    pub fn braces<T>(
        &mut self, 
        f: impl Fn(&mut Self) -> T,
        fail: impl Fn(KupoParseError) -> T,
    ) -> Parse<T> {
        self.grouped(
            f, fail,
            Grouping::LBrace, Grouping::RBrace, 
            "left brace expected", "right brace expected"
        )
    }

    fn grouped<T>(
        &mut self, 
        f: impl Fn(&mut Self) -> T,
        fail: impl Fn(KupoParseError) -> T,
        l: Grouping, r: Grouping, 
        lmsg: &str, rmsg: &str
    ) -> Parse<T> {
        let loc = self.ts.location();
        let _ = if let Some(lbrace) = 
            self.ts.pop_tpred(|t| t == &Token::Grouping(l)) { lbrace } else {
                return self.give_up(lmsg, fail)
            };

        let result = f(self);

        let rbrace = if let Some(rbrace) = 
            self.ts.pop_tpred(|t| t == &Token::Grouping(r)) { rbrace } else {
                return self.give_up(rmsg, fail)
            };

        loc.merge_r(rbrace).replace(result)
    }

    pub fn separated<T>(
        &mut self,

        // peek: if this returns true, we must keep parsing; otherwise break
        // needed because we're allowed to have a terminator that isn't followed by another item
        peek: impl Fn(&mut Self) -> bool,  
        parse: impl Fn(&mut Self) -> T,
        separator: Grouping,
    ) -> Located<Vec<T>> {
        let loc = self.ts.location();
        let mut xs = vec![];

        loop {
            if !peek(self) { break; }

            let x = parse(self);
            xs.push(x);

            let found_terminator = self.ts.pop_tpred(|t| t == &Token::Grouping(separator)).is_some();
            if !found_terminator { break; }
        }

        let loc2 = self.ts.location();
        loc.merge_l(&loc2).replace(xs)
    }

    pub fn many<T>(
        &mut self,

        // peek: if this returns true, we must keep parsing; otherwise break
        // needed because we're allowed to have a terminator that isn't followed by another item
        peek: impl Fn(&mut Self) -> bool,  
        parse: impl Fn(&mut Self) -> T,
    ) -> Located<Vec<T>> {
        let loc = self.ts.location();
        let mut xs = vec![];

        loop {
            if !peek(self) { break; }

            let x = parse(self);
            xs.push(x);
        }

        let loc2 = self.ts.location();
        loc.merge_l(&loc2).replace(xs)
    }

    // brackets are optional for exactly one item
    pub fn brack_comma_sep<T, Ts>(
        &mut self,
        peek: impl Fn(&mut Self) -> bool,
        parse: impl Fn(&mut Self) -> T,
        integrate: impl Fn(Vec<T>) -> Ts,
        fail: impl Fn(KupoParseError) -> Ts,
    ) -> Located<Ts> {
        if self.ts.peek_tpred(|t| t == &Token::Grouping(Grouping::LBrack)) {
            self.brackets(|s| 
                integrate(s.separated(&peek, &parse, Grouping::Comma).value
            ), fail)
        } else {
            let loc1 = self.ts.location();
            let x = parse(self);
            let loc2 = self.ts.location();
            loc1.merge_l(&loc2).replace(integrate(vec![x]))
        }
    }
}