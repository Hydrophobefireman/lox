use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::{
    errors::RuntimeResult,
    interpreter::Interpreter,
    lox_class::{LoxClass, LoxInstance},
    lox_function::LoxFunction,
    tokens::token_type::TokenType,
};
pub trait LoxCallable {
    fn constructor(&self) -> Option<&LoxClass> {
        return None;
    }
    fn kind(&self) -> LoxCallableType;
    fn name(&self) -> String;
    fn arity(&self) -> usize;
    fn call(
        &mut self,
        interpreter: Interpreter,
        args: Vec<LoxType>,
    ) -> RuntimeResult<(LoxType, Interpreter)>;
}
impl std::fmt::Debug for dyn LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} {}()", self.kind(), self.name())
    }
}

#[derive(Debug)]
pub enum LoxCallableType {
    Function,
    Class,
    NativeFunction,
}

// impl Clone for Box<dyn LoxCallable> {
//     fn clone(&self) -> Self {
//         self.clone_box()
//     }
// }

pub enum LoxInstanceValue {
    Free(LoxType),
    Bound(Rc<RefCell<LoxFunction>>),
}
#[derive(Debug, Clone)]
pub enum LoxType {
    String(String),
    Float(f64),
    True,
    False,
    Nil,
    Callable(Rc<RefCell<dyn LoxCallable>>),
    InternalNoValue,
    Data(Rc<RefCell<LoxInstance>>),
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
            LoxType::Callable(c) => write!(f, "[{:?} {}]", c.borrow().kind(), c.borrow().name()),
            LoxType::Data(inst) => write!(f, "{} {{}}", inst.borrow().this.name()),
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
    pub fn dummy_this() -> Self {
        Self::dummy("this", TokenType::This)
    }

    pub fn dummy<T: Into<String>>(x: T, ty: TokenType) -> Self {
        Self {
            ty,
            lexeme: x.into(),
            literal: LoxType::Nil,
            line: 0,
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

pub fn ref_cell<T>(x: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(x))
}
