use std::collections::HashMap;

use crate::program::Program;
use crate::token::Token;
use crate::token_type::TokenType::{self, *};

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    program: &'a Program,
    keywords: HashMap<&'a str, TokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str, p: &'a Program) -> Scanner<'a> {
        let mut keywords: HashMap<&str, TokenType> = HashMap::new();
        keywords.insert("and", And);
        keywords.insert("class", Class);
        keywords.insert("else", Else);
        keywords.insert("false", False);
        keywords.insert("for", For);
        keywords.insert("fun", Fun);
        keywords.insert("if", If);
        keywords.insert("nil", Nil);
        keywords.insert("or", Or);
        keywords.insert("print", Print);
        keywords.insert("return", Return);
        keywords.insert("super", Super);
        keywords.insert("this", This);
        keywords.insert("true", True);
        keywords.insert("var", Var);
        keywords.insert("while", While);
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            program: p,
            keywords,
        }
    }
    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        self.tokens
            .push(Token::new(EOF, "".to_owned(), Some(Box::new(0)), self.line));
        &self.tokens
    }
    #[inline]
    fn curr_char(&self) -> char {
        self.source.as_bytes()[self.current] as char
    }
    #[inline]
    fn advance(&mut self) -> char {
        let res = self.curr_char();
        self.current += 1;
        res as char
    }
    #[inline]
    fn add_token(&mut self, t: TokenType, literal: Option<Box<dyn std::any::Any>>) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(t, text.to_owned(), literal, self.line));
    }
    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(LeftParen, None),
            ')' => self.add_token(RightParen, None),
            '{' => self.add_token(LeftBrace, None),
            '}' => self.add_token(RightBrace, None),
            ',' => self.add_token(Comma, None),
            '.' => self.add_token(Dot, None),
            '-' => self.add_token(Minus, None),
            '+' => self.add_token(Plus, None),
            ';' => self.add_token(Semicolon, None),
            '*' => self.add_token(Star, None),
            '!' => {
                let a = if self.consume_if('=') {
                    BangEqual
                } else {
                    Bang
                };
                self.add_token(a, None)
            }
            '=' => {
                let a = if self.consume_if('=') {
                    EqualEqual
                } else {
                    Equal
                };
                self.add_token(a, None)
            }
            '<' => {
                let a = if self.consume_if('=') {
                    LessEqual
                } else {
                    Less
                };
                self.add_token(a, None)
            }
            '>' => {
                let a = if self.consume_if('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.add_token(a, None)
            }
            '/' => {
                if self.consume_if('/') {
                    loop {
                        if !(self.peek().is_some() && self.peek().unwrap() != '\n') {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(Slash, None);
                }
            }
            '0'..='9' => self.handle_number(),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.handle_string(),
            'a'..='z' | 'A'..='Z' | '_' => self.handle_identifier(),
            _ => self
                .program
                .error(self.line, &format!("Unexpected token: {c}")),
        }
    }
    fn handle_number(&mut self) {
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        if matches!(self.peek(), Some('.')) && matches!(self.peek_next(), Some('0'..='9')) {
            self.advance();
        }
        while matches!(self.peek(), Some('0'..='9')) {
            self.advance();
        }
        self.add_token(
            Number,
            Some(Box::new(
                self.source[self.start..self.current]
                    .parse::<f32>()
                    .unwrap(),
            )),
        )
    }

    fn handle_identifier(&mut self) {
        loop {
            if let Some(x) = self.peek() {
                if x.is_alphanumeric() {
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let text = &self.source[self.start..self.current];
        let tt = self.keywords.get(text);
        if let Some(ttype) = tt {
            self.add_token(*ttype, None);
        } else {
            self.add_token(Identifier, None);
        }
    }
    fn handle_string(&mut self) {
        while self.peek().is_some() && self.peek().unwrap() != '"' {
            if self.peek().unwrap() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.program.error(self.line, "Unterminated string");
        }
        self.advance();

        let value = self.source[(self.start + 1)..(self.current - 1)].to_owned();

        self.add_token(String, Some(Box::new(value)));

        // todo!("add support for escape sequences");
    }

    #[inline]
    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            None
        } else {
            Some(self.source.as_bytes()[self.current + 1] as char)
        }
    }
    #[inline]
    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        return Some(self.curr_char());
    }

    #[inline]
    fn consume_if(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        let c = self.curr_char();
        if c != expected {
            return false;
        }
        self.current += 1;
        return true;
    }
}
