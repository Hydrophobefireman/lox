use std::rc::Rc;

use crate::{
    environment::Environment,
    errors::RuntimeResult,
    interpreter::Interpreter,
    syntax::stmt::Function,
    tokens::token::{LoxCallable, LoxCollableType, LoxType},
};

pub struct LoxFunction {
    declaration: Function,
}

impl LoxFunction {
    pub fn new(declaration: Function) -> Self {
        Self { declaration }
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
        Box::new(Self::new(self.declaration.clone()))
    }
    fn call(&mut self, interpreter: &mut Interpreter, args: Vec<LoxType>) -> RuntimeResult<LoxType> {
        let mut env = Environment::new(Some(Rc::clone(&interpreter.globals)));

        self.declaration
            .params
            .iter()
            .zip(&args)
            .for_each(|(param, arg)| {
                env.define(&param.lexeme, arg.clone());
            });
        interpreter.execute_block(&self.declaration.body, env)?;
        Ok(Default::default())
    }
}
impl From<LoxFunction> for LoxType {
    fn from(value: LoxFunction) -> Self {
        LoxType::Callable(Box::new(value))
    }
}