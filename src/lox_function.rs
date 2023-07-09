use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::Environment,
    errors::{InterruptKind, RuntimeResult},
    interpreter::Interpreter,
    syntax::stmt::Function,
    tokens::token::{LoxCallable, LoxCollableType, LoxType},
};

pub struct LoxFunction {
    declaration: Function,
    closure: Rc<RefCell<Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: Function, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            declaration,
            closure,
        }
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
        ))
    }
    fn call(
        &mut self,
        interpreter: &mut Interpreter,
        args: Vec<LoxType>,
    ) -> RuntimeResult<LoxType> {
        let mut env = Environment::new(Some(Rc::clone(&self.closure)));

        self.declaration
            .params
            .iter()
            .zip(&args)
            .for_each(|(param, arg)| {
                env.define(&param.lexeme, arg.clone());
            });
        match interpreter.execute_block(&self.declaration.body, env) {
            Err(err) => match err.interrupt_kind {
                InterruptKind::Builtin => Err(err),
                InterruptKind::Return(val) => Ok(val),
            },
            _ => Ok(LoxType::Nil.into()),
        }
    }
}
impl From<LoxFunction> for LoxType {
    fn from(value: LoxFunction) -> Self {
        LoxType::Callable(Box::new(value))
    }
}
