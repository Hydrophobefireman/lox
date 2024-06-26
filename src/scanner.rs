use crate::errors::{ScanError, ScanResult};
use crate::tokens::token::{LoxType, Token};
use crate::tokens::token_type::TokenType::{self};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    pub fn scan_tokens(mut self) -> ScanResult<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token::new(
            TokenType::EOF,
            "".to_owned(),
            LoxType::InternalNoValue,
            self.line,
        ));
        Ok(self.tokens)
    }
    #[inline]
    fn curr_char(&self) -> char {
        self.source.as_bytes()[self.current] as char
    }
    #[inline]
    fn advance(&mut self) -> char {
        let res = self.curr_char();
        self.current += 1;
        return res as char;
    }
    #[inline]
    fn add_token(&mut self, t: TokenType, literal: LoxType) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(t, text.to_owned(), literal, self.line));
    }
    fn scan_token(&mut self) -> ScanResult<()> {
        use TokenType::*;
        let c = self.advance();
        match c {
            '(' => self.add_token(LeftParen, LoxType::InternalNoValue),
            ')' => self.add_token(RightParen, LoxType::InternalNoValue),
            '{' => self.add_token(LeftBrace, LoxType::InternalNoValue),
            '}' => self.add_token(RightBrace, LoxType::InternalNoValue),
            ',' => self.add_token(Comma, LoxType::InternalNoValue),
            '.' => self.add_token(Dot, LoxType::InternalNoValue),
            '-' => self.add_token(Minus, LoxType::InternalNoValue),
            '+' => self.add_token(Plus, LoxType::InternalNoValue),
            ';' => self.add_token(Semicolon, LoxType::InternalNoValue),
            '*' => self.add_token(Star, LoxType::InternalNoValue),
            '!' => {
                let a = if self.consume_if('=') {
                    BangEqual
                } else {
                    Bang
                };
                self.add_token(a, LoxType::InternalNoValue)
            }
            '=' => {
                let a = if self.consume_if('=') {
                    EqualEqual
                } else {
                    Equal
                };
                self.add_token(a, LoxType::InternalNoValue)
            }
            '<' => {
                let a = if self.consume_if('=') {
                    LessEqual
                } else {
                    Less
                };
                self.add_token(a, LoxType::InternalNoValue)
            }
            '>' => {
                let a = if self.consume_if('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.add_token(a, LoxType::InternalNoValue)
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
                    self.add_token(Slash, LoxType::InternalNoValue);
                }
            }
            '0'..='9' => self.handle_number(),
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.handle_string()?,
            'a'..='z' | 'A'..='Z' | '_' => self.handle_identifier(),
            _ => Err(ScanError::new(format!("Unexpected token: {c}"), self.line))?,
        };
        Ok(())
    }
    fn handle_number(&mut self) {
        use TokenType::*;
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
            LoxType::Float(
                self.source[self.start..self.current]
                    .parse::<f64>()
                    .unwrap(),
            ),
        )
    }

    fn handle_identifier(&mut self) {
        use TokenType::*;
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
        let tt = match text {
            "and" => And,
            "class" => Class,
            "else" => Else,
            "false" => False,
            "for" => For,
            "fun" => Fun,
            "if" => If,
            "nil" => Nil,
            "or" => Or,
            "print" => Print,
            "return" => Return,
            "super" => Super,
            "this" => This,
            "true" => True,
            "var" => Var,
            "while" => While,
            _ => Identifier,
        };
        self.add_token(tt, LoxType::InternalNoValue);
    }
    fn handle_string(&mut self) -> ScanResult<()> {
        use TokenType::*;
        while self.peek().is_some() && self.peek().unwrap() != '"' {
            if self.peek().unwrap() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(ScanError::new("Unterminated string", self.line));
        }
        self.advance();

        let value = self.source[(self.start + 1)..(self.current - 1)].to_owned();

        self.add_token(String, LoxType::String(value));
        Ok(())
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
