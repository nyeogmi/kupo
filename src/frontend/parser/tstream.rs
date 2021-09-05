use crate::frontend::{lexer::Token, located::Located};

pub struct TStream<'a> {
    tokens: &'a [Located<Token>],
    eof: Located<Token>,
}

impl<'a> TStream<'a> {
    pub fn new(tokens: &'a [Located<Token>], eof: Located<()>) -> Self {
        TStream { tokens, eof: eof.replace(Token::EOF) }
    }

    pub fn location(&self) -> Located<()> {
        let mut loc = self.tokens.iter().nth(0).map_or(self.eof.location(), |x| x.location());
        loc.end = loc.start;
        return loc;
    }

    pub fn advance(&mut self, amt: usize) {
        if !self.tokens.is_empty() { 
            self.tokens = &self.tokens[amt..];
        }
    }

    pub fn pop_tpred(&mut self, f: impl Fn(&Token) -> bool) -> Option<Located<Token>> {
        let t = if let Some(t) = self.tokens.iter().nth(0) {
            t
        } else { &self.eof };

        if f(&t.value) {
            let result = Some(t.clone());
            self.advance(1);
            return result
        }
        None
    }

    pub fn peek_tpred(&self, f: impl Fn(&Token) -> bool) -> bool {
        let t = if let Some(t) = self.tokens.iter().nth(0) { t } 
        else { &self.eof };

        f(&t.value)
    }

    pub fn pop_eq(&mut self, t: &Token) -> Option<Located<Token>> {
        self.pop_tpred(|t2| t2 == t)
    }

    pub fn pop_keyword(&mut self, s: &str) -> Option<Located<Token>> {
        self.pop_tpred(|t| match t { 
            Token::Keyword(i) if i == s => true,
            _ => false,
        })
    }

    pub fn pop_identifier(&mut self) -> Option<Located<String>> {
        // TODO: Assert that it's not a keyword 
        self.pop_tpred(|t| match t { 
            Token::Identifier(_) => true,
            _ => false,
        }).map(|o| o.locmap(|t| 
            match t {
                Token::Identifier(v) => v,
                _ => unreachable!()
            }
        ))
    }

    pub fn pop_variable(&mut self) -> Option<Located<String>> {
        // TODO: Assert that it's not a keyword 
        self.pop_tpred(|t| match t { 
            Token::Variable(_) => true,
            _ => false,
        }).map(|o| o.locmap(|t| 
            match t {
                Token::Variable(v) => v,
                _ => unreachable!()
            }
        ))
    }

    pub fn pop_any(&mut self) -> Located<Token> {
        self.pop_tpred(|_| true).unwrap()
    }

    pub fn peek_eq(&self, t: &Token) -> bool {
        self.peek_tpred(|t2| t2 == t)
    }
    
    pub fn peek_keyword(&self, s: &str) -> bool {
        self.peek_tpred(|t| match t { 
            Token::Keyword(i) if i == s => true,
            _ => false,
        })
    }

    pub fn peek_identifier(&self) -> bool {
        self.peek_tpred(|t| match t { 
            Token::Identifier(_) => true,
            _ => false,
        })
    }

    pub fn peek_variable(&self) -> bool {
        self.peek_tpred(|t| match t { 
            Token::Variable(_) => true,
            _ => false,
        })
    }

    pub fn peek_any(&self) -> &Located<Token> {
        if self.tokens.is_empty() { &self.eof }
        else { &self.tokens[0] }
    }
}
