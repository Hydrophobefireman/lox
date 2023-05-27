use std::error::Error;

use crate::{errors::RuntimeError, tokens::token_type::TokenType};

#[derive(Debug, Clone)]
pub enum LiteralType {
    String(String),
    Float(f64),
    True,
    False,
    Nil,
    None,
}

impl From<bool> for LiteralType {
    fn from(value: bool) -> Self {
        return if value {
            LiteralType::True
        } else {
            LiteralType::False
        };
    }
}
pub fn literal_to_float(x: LiteralType) -> Result<f64, RuntimeError> {
    match x {
        LiteralType::Float(v) => Ok(v),
        _ => Err(RuntimeError::new("Cannot convert to float")),
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: String,
    pub literal: LiteralType,
    pub line: usize,
}

impl Token {
    pub fn new(ty: TokenType, lexeme: String, literal: LiteralType, line: usize) -> Self {
        Token {
            ty,
            lexeme,
            literal,
            line,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {:?} {}",
            self.ty, self.lexeme, self.literal, self.line
        )
    }
}
