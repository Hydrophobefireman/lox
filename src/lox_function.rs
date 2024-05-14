use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    errors::{InterruptKind, RuntimeError, RuntimeResult},
    interpreter::Interpreter,
    syntax::stmt::Function,
    tokens::{
        token::{LoxCallable, LoxCollableType, LoxType, Token},
        token_type::TokenType,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum FunctionKind {
    Init,
    Function,
}
#[derive(Debug, Clone)]
pub struct LoxFunction {
    declaration: Function,
    closure: Rc<RefCell<Environment>>,
    kind: FunctionKind,
}

impl LoxFunction {
    pub fn new(
        declaration: Function,
        closure: Rc<RefCell<Environment>>,
        kind: FunctionKind,
    ) -> Self {
        Self {
            declaration,
            closure,
            kind,
        }
    }
    pub fn bind(&self, to: LoxType) -> Self {
        let k = self.kind;
        if !matches!(to, LoxType::Data(_)) {
            panic!("Cannot bind function to a non instance object!")
        }
        let mut env = Environment::new(Some(Rc::clone(&self.closure)));
        env.define("this", to);
        return Self::new(self.declaration.clone(), Rc::new(RefCell::new(env)), k);
    }
}

impl LoxCallable for LoxFunction {
    fn kind(&self) -> LoxCollableType {
        LoxCollableType::Function
    }
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }
    fn name(&self) -> String {
        return self.declaration.name.lexeme.clone();
    }
    fn clone_box(&self) -> Box<dyn LoxCallable> {
        Box::new(Self::new(
            self.declaration.clone(),
            Rc::clone(&self.closure),
            self.kind,
        ))
    }
    fn call(
        &mut self,
        interpreter: Interpreter,
        args: Vec<LoxType>,
    ) -> RuntimeResult<(LoxType, Interpreter)> {
        let mut env = Environment::new(Some(Rc::clone(&self.closure)));

        self.declaration
            .params
            .iter()
            .zip(&args)
            .for_each(|(param, arg)| {
                env.define(&param.lexeme, arg.clone());
            });
        match interpreter.execute_block(self.declaration.body.clone(), env) {
            Err(err) => match err.interrupt_kind {
                InterruptKind::Builtin => Err(err),
                InterruptKind::Return(val) => {
                    if matches!(self.kind, FunctionKind::Init) {
                        return match self.closure.borrow().get_at(&Token::dummy_this(), 0) {
                            Err(e) => Err(RuntimeError::new(e.message, e.line, err.interpreter)),
                            Ok(this) => Ok((this, err.interpreter)),
                        };
                    }
                    Ok((val, err.interpreter))
                }
            },
            Ok(ret) => match self.kind {
                FunctionKind::Init => match self.closure.borrow().get_at(&Token::dummy_this(), 0) {
                    Err(e) => Err(RuntimeError::new(e.message, e.line, ret)),
                    Ok(val) => Ok((val, ret)),
                },
                _ => Ok((LoxType::Nil.into(), ret)),
            },
        }
    }
}
impl From<LoxFunction> for LoxType {
    fn from(value: LoxFunction) -> Self {
        LoxType::Callable(Box::new(value))
    }
}
