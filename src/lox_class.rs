use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{LoxClassError, LoxClassResult, RuntimeResult},
    interpreter::Interpreter,
    lox_function::LoxFunction,
    tokens::token::{LoxCallable, LoxCollableType, LoxInstanceValue, LoxType, Token},
};

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, LoxFunction>,
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    pub this: LoxClass,
    fields: HashMap<String, LoxType>,
}

impl LoxInstance {
    pub fn new(this: LoxClass) -> Self {
        Self {
            this,
            fields: Default::default(),
        }
    }
    pub fn get(&self, name: Token) -> LoxClassResult<LoxInstanceValue> {
        match self.fields.get(&name.lexeme) {
            Some(val) => Ok(LoxInstanceValue::Free(val.clone().into())),
            None => match self.this.find_method(&name.lexeme) {
                Some(val) => Ok(LoxInstanceValue::Bound(val)),
                None => Err(LoxClassError::new(
                    format!("Undefined property '{}'.", &name.lexeme),
                    name.line,
                )),
            },
        }
    }
    pub fn set(&mut self, name: Token, value: LoxType) -> () {
        self.fields.insert(name.lexeme, value);
    }
}

impl LoxClass {
    pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
        Self { name, methods }
    }
    fn find_method(&self, method: &String) -> Option<&LoxFunction> {
        self.methods.get(method)
    }
}

impl From<LoxClass> for LoxType {
    fn from(value: LoxClass) -> Self {
        LoxType::Callable(Box::new(value))
    }
}
impl From<LoxInstance> for LoxType {
    fn from(value: LoxInstance) -> Self {
        LoxType::Data(Rc::new(RefCell::new(value)))
    }
}

impl LoxCallable for LoxClass {
    fn kind(&self) -> LoxCollableType {
        LoxCollableType::Class
    }
    fn arity(&self) -> usize {
        0
    }
    fn name(&self) -> String {
        return self.name.clone();
    }
    fn clone_box(&self) -> Box<dyn LoxCallable> {
        Box::new(self.clone())
    }
    fn call(
        &mut self,
        interpreter: Interpreter,
        args: Vec<LoxType>,
    ) -> RuntimeResult<(LoxType, Interpreter)> {
        let inst = LoxInstance::new(self.clone());
        if let Some(initializer) = self.find_method(&"init".into()) {
            let (res, interp) = initializer.bind(inst.into()).call(interpreter, args)?;
            return Ok((res, interp));
        }
        Ok((inst.into(), interpreter))
    }
}
