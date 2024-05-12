use crate::{interpreter::Interpreter, resolver::Resolver, tokens::token::LoxType};

#[derive(Debug)]
pub enum InterruptKind {
    Builtin,
    Return(LoxType),
}

macro_rules! err_struct {
    ($name:ident,$err:ident) => {
        #[derive(Debug)]
        pub struct $name {
            pub message: String,
            pub line: usize,
            pub interrupt_kind: InterruptKind,
        }
        impl $name {
            pub fn new(message: String, line: usize) -> Self {
                Self {
                    message,
                    line,
                    interrupt_kind: InterruptKind::Builtin,
                }
            }
        }
        pub type $err<T> = Result<T, $name>;
    };
}

err_struct!(RuntimeError, RuntimeResult);

err_struct!(ScanError, ScanResult);

err_struct!(ParseError, ParseResult);

#[derive(Debug)]
pub struct ResolverError {
    pub message: String,
    pub line: usize,
    pub interrupt_kind: InterruptKind,
    pub interpreter: Interpreter,
}
impl ResolverError {
    pub fn new(message: String, line: usize, i: Interpreter) -> Self {
        Self {
            message,
            line,
            interrupt_kind: InterruptKind::Builtin,
            interpreter: i,
        }
    }
}
pub type ResolverResult<T> = Result<(T, Resolver), ResolverError>;

impl RuntimeError {
    pub fn as_return(value: LoxType) -> Self {
        Self {
            message: Default::default(),
            line: 0,
            interrupt_kind: InterruptKind::Return(value),
        }
    }
}
