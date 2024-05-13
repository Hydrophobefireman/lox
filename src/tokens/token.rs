use std::fmt::Display;

use crate::{errors::RuntimeResult, interpreter::Interpreter, tokens::token_type::TokenType};
pub trait LoxCallable {
    fn kind(&self) -> LoxCollableType;
    fn name(&self) -> String;
    fn arity(&self) -> usize;
    fn call(
        &mut self,
        interpreter: Interpreter,
        args: Vec<LoxType>,
    ) -> RuntimeResult<(LoxType, Interpreter)>;
    fn clone_box(&self) -> Box<dyn LoxCallable>;
}
impl std::fmt::Debug for dyn LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} {}()", self.kind(), self.name())
    }
}

#[derive(Debug)]
pub enum LoxCollableType {
    Function,
    Class,
    NativeFunction,
}

impl Clone for Box<dyn LoxCallable> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
#[derive(Debug, Clone)]
pub enum LoxType {
    String(String),
    Float(f64),
    True,
    False,
    Nil,
    Callable(Box<dyn LoxCallable>),
    InternalNoValue,
}

impl PartialEq for LoxType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            (LoxType::True, LoxType::True)
            | (LoxType::False, LoxType::False)
            | (LoxType::InternalNoValue, LoxType::InternalNoValue) => true,
            _ => false,
        }
    }
}

impl Display for LoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxType::String(s) => write!(f, "{s}"),
            LoxType::Float(n) => write!(f, "{n}"),
            LoxType::True => write!(f, "true"),
            LoxType::False => write!(f, "false"),
            LoxType::Nil => write!(f, "nil"),
            LoxType::InternalNoValue => write!(f, "(?unresolved?)"),
            LoxType::Callable(c) => write!(f, "[{:?} {}]", c.kind(), &c.name()),
        }
    }
}
impl From<bool> for LoxType {
    fn from(value: bool) -> Self {
        return if value { LoxType::True } else { LoxType::False };
    }
}

impl From<f64> for LoxType {
    fn from(value: f64) -> Self {
        LoxType::Float(value)
    }
}

impl Default for LoxType {
    fn default() -> Self {
        LoxType::InternalNoValue
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: TokenType,
    pub lexeme: String,
    pub literal: LoxType,
    pub line: usize,
}

impl Token {
    pub fn new(ty: TokenType, lexeme: String, literal: LoxType, line: usize) -> Self {
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
