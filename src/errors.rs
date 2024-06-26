use crate::{interpreter::Interpreter, resolver::Resolver, tokens::token::LoxType};

#[derive(Debug)]
pub enum InterruptKind {
    Builtin,
    Return(LoxType),
}

#[derive(Debug)]
pub struct ResolverError {
    pub message: String,
    pub line: usize,
    pub interrupt_kind: InterruptKind,
    pub interpreter: Interpreter,
}
impl ResolverError {
    pub fn new<T: Into<String>>(message: T, line: usize, i: Interpreter) -> Self {
        Self {
            message: message.into(),
            line,
            interrupt_kind: InterruptKind::Builtin,
            interpreter: i,
        }
    }
}
pub type ResolverResult<T> = Result<(T, Resolver), ResolverError>;

impl RuntimeError {
    pub fn as_return(value: LoxType, this: Interpreter) -> Self {
        Self {
            message: Default::default(),
            line: 0,
            interrupt_kind: InterruptKind::Return(value),
            interpreter: this,
        }
    }
}
#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub line: usize,
    pub interrupt_kind: InterruptKind,
    pub interpreter: Interpreter,
}
impl RuntimeError {
    pub fn new<T: Into<String>>(message: T, line: usize, i: Interpreter) -> Self {
        Self {
            message: message.into(),
            line,
            interrupt_kind: InterruptKind::Builtin,
            interpreter: i,
        }
    }
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

macro_rules! err_struct {
    ($name:ident,$err:ident) => {
        #[derive(Debug)]
        pub struct $name {
            pub message: String,
            pub line: usize,
            pub interrupt_kind: InterruptKind,
        }
        impl $name {
            pub fn new<T: Into<String>>(message: T, line: usize) -> Self {
                Self {
                    message: message.into(),
                    line,
                    interrupt_kind: InterruptKind::Builtin,
                }
            }
        }
        pub type $err<T> = Result<T, $name>;
    };
}

err_struct!(ScanError, ScanResult);

err_struct!(ParseError, ParseResult);
err_struct!(EnvError, EnvResult);
err_struct!(LoxClassError, LoxClassResult);
