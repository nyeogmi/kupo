use super::keywords::is_keyword;

use super::located::Located;

use lazy_static::lazy_static;
use regex::Regex;
use std::char;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    Invalid(Invalid),
    Keyword(String), 
    Identifier(String), 
    Variable(String), 
    Integer(u64),
    // Float(f64),
    StringLiteral(String),
    Grouping(Grouping),
    Operator(Operator),
    EOF,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grouping {
    LParen, RParen, LBrace, RBrace, LBrack, RBrack,
    Comma, Semicolon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    OAdd, OSubtract, OMultiply, ODivide, ODot,
    OAssign, OAssignNew,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Invalid {
    Char(char),
    StringLiteral(usize, String),  // error position, error
    Integer(String),
}

struct CStream<'a> {
    s: &'a str,
    offset: usize,
}

impl<'a> CStream<'a> {
    fn new(s: &'a str) -> Self {
        CStream { s, offset: 0 }
    }

    fn advance(&mut self, amt: usize) {
        self.s = &self.s[amt..];
        self.offset += amt;
    }

    fn any(&self) -> bool {
        self.s.len() > 0
    }

    fn pop_string(&mut self, s: &str) -> bool {
        if self.s.len() < s.len() {
            return false;
        }
        for (c0, c1) in self.s.chars().zip(s.chars()) {
            if c0 == c1 { continue; }
            return false;
        }
        self.advance(s.len());
        return true;
    }

    fn pop_cpred(&mut self, f: impl Fn(char) -> bool) -> Option<char> {
        if let Some(c) = self.s.chars().nth(0) {
            if f(c) {
                self.advance(c.len_utf8());
                return Some(c);
            }
        }
        return None;
    }

    fn pop_char(&mut self, c: char) -> bool { self.pop_cpred(|c2| c2 == c).is_some() }

    fn pop_ws(&mut self) -> bool {
        // TODO: Use unicode
        self.pop_cpred(|c| " \t\n\r".contains(c)).is_some()
    }

    fn pop_regex(&mut self, r: &Regex) -> Option<&'a str> {
        if let Some(mtch) = r.find(self.s) {
            assert_eq!(mtch.start(), 0);
            let match_part = &self.s[..mtch.end()];
            self.advance(mtch.end());
            return Some(match_part);
        }
        None
    }

    fn pop_any(&mut self) -> Option<char> {
        self.pop_cpred(|_| true)
    }
}

struct Lexer<'a> {
    cs: CStream<'a>,
    tokens: Vec<Located<Token>>,
}

pub fn lex(s: &str) -> (Vec<Located<Token>>, Located<()>) {
    let cs = CStream::new(s);
    let tokens = vec![];

    Lexer { cs, tokens }.lex()
}

impl<'a> Lexer<'a> {
    fn lex(mut self) -> (Vec<Located<Token>>, Located<()>) {
        while self.cs.any() {
            if self.whitespace() { continue; }
            if self.singleline_comment() { continue; }
            if self.identifier() { continue; }
            if self.variable() { continue; }
            // if self.float() { continue; }
            if self.integer() { continue; }
            if self.dq_string_literal() { continue; }
            if self.sq_string_literal() { continue; }
            if self.grouping() { continue; }
            if self.operator() { continue; }

            // println!("wtf is this");
            let start = self.cs.offset;
            if let Some(c) = self.cs.pop_any() {
                let end = self.cs.offset;
                self.tokens.push(Located { start, end, value: Token::Invalid(Invalid::Char(c)) });
            }
            // TODO: Add an invalid token
        }
        (self.tokens, Located { start: self.cs.offset, end: self.cs.offset, value: () })
    }

    fn whitespace(&mut self) -> bool {
        let mut any = false;
        while self.cs.pop_ws() {
            any = true;
        }
        return any;
    }

    fn singleline_comment(&mut self) -> bool {
        if !self.cs.pop_char('#') {
            return false
        }
        while let Some(c) = self.cs.pop_any() {
            if c == '\n' { break; }
        }
        true
    }

    fn variable(&mut self) -> bool {
        let start = self.cs.offset;
        lazy_static! {
            static ref RE: Regex = Regex::new("\\A@[a-zA-Z][_a-zA-Z0-9]*").unwrap();
        }
        if let Some(ident) = self.cs.pop_regex(&RE) {
            let end = self.cs.offset;
            self.tokens.push(Located { start, end, value: Token::Variable(ident.to_owned()) });
            return true
        }
        return false
    }

    fn identifier(&mut self) -> bool {
        let start = self.cs.offset;
        lazy_static! {
            static ref RE: Regex = Regex::new("\\A[a-zA-Z][_a-zA-Z0-9]*").unwrap();
        }
        if let Some(ident) = self.cs.pop_regex(&RE) {
            let end = self.cs.offset;
            self.tokens.push(Located { start, end, value: 
                if is_keyword(ident) { Token::Keyword(ident.to_owned()) }
                else { Token::Identifier(ident.to_owned()) }
            });
            return true
        }
        return false
    }

    /*
    fn float(&mut self) -> bool {
        // TODO: Prevent the ".1" production from being used right after an identifier. 
        // Perhaps make operators take higher precedence, but only enable the `.` production right after an identifier
        let start = self.cs.offset;
        lazy_static! {
            static ref RE: Regex = Regex::new("\\A([0-9][0-9_]*\\.([0-9][0-9_]*)?|\\.[0-9][0-9_]*)").unwrap();
        }
    }
    */

    fn integer(&mut self) -> bool {
        let start = self.cs.offset;
        lazy_static! {
            static ref RE: Regex = Regex::new("\\A([0-9][0-9]*|0x[0-9a-fA-F][0-9a-fA-F_]*|0o[0-7][0-7]*|0d[0-9][0-9]*|0b[01][01_]*)\\b").unwrap();
        }

        if let Some(integer) = self.cs.pop_regex(&RE) {
            let i2 = integer.to_owned().replace("_", ""); // TODO: Don't waste an allocation
            let end = self.cs.offset;

            let expr: Result<u64, _> = 
                if i2.starts_with("0x") {
                    u64::from_str_radix(&i2[2..], 16)
                } else if i2.starts_with("0o") {
                    u64::from_str_radix(&i2[2..], 8)
                } else if i2.starts_with("0d") {
                    u64::from_str_radix(&i2[2..], 10)
                } else if i2.starts_with("0b") {
                    u64::from_str_radix(&i2[2..], 2)
                } else {
                    u64::from_str_radix(&i2, 10)
                };

            match expr {
                Ok(int) => self.tokens.push(Located { start, end, value: Token::Integer(int) } ),
                Err(e) => {
                    // invalid integer (probably too big)
                    self.tokens.push(Located { start, end, value: Token::Invalid(
                        Invalid::Integer(format!("invalid integer: {} ({})", integer, e))
                    )})
                }
            }
            return true
        }
        return false
    }

    fn grouping(&mut self) -> bool {
        let start = self.cs.offset;

        let g: Grouping;
        if self.cs.pop_string("(") { g = Grouping::LParen; }
        else if self.cs.pop_string(")") { g = Grouping::RParen; }
        else if self.cs.pop_string("{") { g = Grouping::LBrace; }
        else if self.cs.pop_string("}") { g = Grouping::RBrace; }
        else if self.cs.pop_string("[") { g = Grouping::LBrack; }
        else if self.cs.pop_string("]") { g = Grouping::RBrack; }
        else if self.cs.pop_string(",") { g = Grouping::Comma; }
        else if self.cs.pop_string(";") { g = Grouping::Semicolon; }
        else { return false; }

        let end = self.cs.offset;

        self.tokens.push(Located { start, end, value: Token::Grouping(g) });
        return true;
    }

    fn operator(&mut self) -> bool {
        let start = self.cs.offset;

        let op: Operator;
        if self.cs.pop_string("+") { op = Operator::OAdd; }
        else if self.cs.pop_string("-") { op = Operator::OSubtract; }
        else if self.cs.pop_string("*") { op = Operator::OMultiply; }
        else if self.cs.pop_string("/") { op = Operator::ODivide; }
        else if self.cs.pop_string(".") { op = Operator::ODot; }
        else if self.cs.pop_string(":=") { op = Operator::OAssignNew; }
        // else if self.cs.pop_string("=") { op = Operator::OAssign; }
        else { return false; }

        let end = self.cs.offset;

        self.tokens.push(Located { start, end, value: Token::Operator(op) });
        return true;
    }

    fn dq_string_literal(&mut self) -> bool {
        self.string_literal('\"')
    }

    fn sq_string_literal(&mut self) -> bool {
        self.string_literal('\'')
    }

    fn string_literal(&mut self, terminator: char) -> bool {
        let start = self.cs.offset;

        if !self.cs.pop_char(terminator) { return false; }

        let mut s = String::new();
        let mut poison: Option<(usize, String)> = None;

        loop {
            let mut poison_ix = self.cs.offset;
            let mbc = self.cs.pop_any();
            let c: char;

            if let Some(c2) = mbc { c = c2;}
            // TODO: "invalid string" token
            else { poison = poison.or(Some((poison_ix, "EOF in string".to_owned()))); break; }
            
            if c == terminator {
                break; 
            }
            if c == '\n' {
                // TODO: "invalid string" token, and break now
                poison = poison.or(Some((poison_ix, "newline in string".to_owned())));
                break;
            }

            if c == '\\' {
                poison_ix = self.cs.offset;

                lazy_static! {
                    static ref CCODE0: Regex = Regex::new("\\A(x[0-9a-fA-F]{2}|u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8})").unwrap();
                }

                if self.cs.pop_char(terminator) { s.push(terminator) }
                else if self.cs.pop_char('\\') { s.push('\\') }
                else if self.cs.pop_char('n') { s.push('\n') }
                else if self.cs.pop_char('r') { s.push('\r') }
                else if self.cs.pop_char('t') { s.push('\t') }
                else if let Some(ccode) = self.cs.pop_regex(&CCODE0) {
                    let cval: Result<u32, _>;
                    match ccode.chars().nth(0) {
                        Some('x') => {
                            assert_eq!(ccode.len(), 3);
                            cval = u32::from_str_radix(&ccode[1..3], 16)
                        }
                        Some('u') => {
                            assert_eq!(ccode.len(), 5);
                            cval = u32::from_str_radix(&ccode[1..5], 16)
                        }
                        Some('U') => {
                            assert_eq!(ccode.len(), 9);
                            cval = u32::from_str_radix(&ccode[1..9], 16)
                        }
                        _ => {
                            panic!("invalid code");
                        }
                    }
                    match cval {
                        Ok(code) => match std::char::from_u32(code) {
                            Some(c) => s.push(c),
                            _ => { 
                                poison = poison.or(Some((poison_ix, format!("character code doesn't map to valid unicode character: {}", code))));
                            }
                        }
                        _ => {
                            poison = poison.or(Some((poison_ix, format!("character code didn't parse: {}", ccode))));
                        }
                    }
                } else if self.cs.pop_char('x') || self.cs.pop_char('u') || self.cs.pop_char('U') { 
                    poison = poison.or(Some((poison_ix, format!("malformatted character code escape sequence"))))
                } else {
                    poison = poison.or(Some((poison_ix, match self.cs.pop_any() {
                        None => format!("EOF in escape sequence"),
                        Some(c) => format!("unrecognized escape sequence: {}", c),
                    })))
                }
            } else {
                s.push(c)
            }
        }

        let end = self.cs.offset;
        match poison {
            None => self.tokens.push(Located { start, end, value: Token::StringLiteral(s) }),
            Some((ix, err)) => self.tokens.push(Located {
                start, end, 
                value: Token::Invalid(Invalid::StringLiteral(ix, err))
            }),
        }
        true
    }

}