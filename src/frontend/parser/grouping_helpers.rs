use crate::frontend::{lexer::{Grouping, Token}, located::Located, parser::error_helpers::kpe};

use super::{Parser, internal_ast::KupoParseError};

pub struct DelimitedMany {
    pub can_be_bare: bool,
    pub consume_rhs: bool,
    pub lhs: Option<(Token, &'static str)>,
    pub rhs: (Token, &'static str),
    pub separator: Option<Token>,
}

impl DelimitedMany {
    pub fn parens_basis() -> Self {
        DelimitedMany {
            can_be_bare: false,
            consume_rhs: true,
            lhs: Some((Token::Grouping(Grouping::LParen), "left paren expected")),
            rhs: (Token::Grouping(Grouping::RParen), "right paren expected"),
            separator: None,
        }
    }

    pub fn brackets_basis() -> Self {
        DelimitedMany {
            can_be_bare: false,
            consume_rhs: true,
            lhs: Some((Token::Grouping(Grouping::LBrack), "left bracket expected")),
            rhs: (Token::Grouping(Grouping::RBrack), "right bracket expected"),
            separator: None,
        }
    }

    pub fn braces_basis() -> Self {
        DelimitedMany {
            can_be_bare: false,
            consume_rhs: true,
            lhs: Some((Token::Grouping(Grouping::LBrace), "left bracket expected")),
            rhs: (Token::Grouping(Grouping::RBrace), "right bracket expected"),
            separator: None,
        }
    }
}

impl<'a> Parser<'a> {
    pub fn located<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> Located<T> {
        let loc1 = self.ts.location();
        let t = f(self);
        let loc2 = self.ts.location();
        loc1.merge_l(&loc2).replace(t)
    }

    pub fn group<T: std::fmt::Debug, Ts>(
        &mut self,
        rules: DelimitedMany,
        parse: impl Fn(&mut Self) -> T,
        integrate: impl Fn(Vec<T>) -> Ts,
        fail: impl Fn(KupoParseError) -> Ts,
    ) -> Located<Ts> {
        return self.located(|s| {
            if let Some((lhs, lmsg)) = rules.lhs {
                if s.ts.pop_eq(&lhs).is_none() {
                    if rules.can_be_bare {
                        return integrate(vec![parse(s)]);
                    }
                    return fail(kpe(lmsg));
                }
            }

            let mut xs = vec![];

            loop {
                if s.ts.peek_eq(&rules.rhs.0) { break; }
                if s.ts.peek_eq(&Token::EOF) { return fail(kpe("found EOF before end of group")); }

                let x = parse(s);
                xs.push(x);

                if let Some(sep) = &rules.separator {
                    let found_terminator = s.ts.pop_eq(&sep).is_some();
                    if !found_terminator { 
                        // println!("breaking: did not find: {:?}", sep);
                        break; 
                    }
                }
            }

            let (rhs, rmsg) = rules.rhs;
            if rules.consume_rhs {
                // println!("looking for: {:?}", rhs);
                if s.ts.pop_eq(&rhs).is_none() {
                    // println!("did not find it");
                    return fail(kpe(rmsg));
                };
            } else {
                if !s.ts.peek_eq(&rhs) {
                    return fail(kpe(rmsg));
                };
            }

            integrate(xs)
        })
    }
}