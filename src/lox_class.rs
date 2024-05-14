use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{LoxClassError, LoxClassResult, RuntimeResult},
    interpreter::Interpreter,
    lox_function::LoxFunction,
    tokens::token::{LoxCallable, LoxCallableType, LoxInstanceValue, LoxType, Token},
};

pub type SuperClass = Option<Rc<RefCell<LoxClass>>>;

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, LoxFunction>,
    superclass: SuperClass,
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    pub this: LoxClass,
    fields: HashMap<String, LoxType>,
}

impl LoxClass {
    pub fn new(
        name: String,
        methods: HashMap<String, LoxFunction>,
        superclass: SuperClass,
    ) -> Self {
        Self {
            name,
            methods,
            superclass,
        }
    }
    fn find_method<T: Into<String>>(&self, method: T) -> Option<&LoxFunction> {
        self.methods.get(&method.into())
    }
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

impl LoxCallable for LoxClass {
    fn kind(&self) -> LoxCallableType {
        LoxCallableType::Class
    }
    fn arity(&self) -> usize {
        match self.find_method("init") {
            None => 0,
            Some(init) => init.arity(),
        }
    }
    fn name(&self) -> String {
        return self.name.clone();
    }
    // fn clone_box(&self) -> Box<dyn LoxCallable> {
    //     Box::new(self.clone())
    // }
    fn call(
        &mut self,
        interpreter: Interpreter,
        args: Vec<LoxType>,
    ) -> RuntimeResult<(LoxType, Interpreter)> {
        let inst = LoxInstance::new(self.clone());
        if let Some(initializer) = self.find_method("init") {
            let (res, interp) = initializer.bind(inst.into()).call(interpreter, args)?;
            return Ok((res, interp));
        }
        Ok((inst.into(), interpreter))
    }
}

impl From<LoxClass> for LoxType {
    fn from(value: LoxClass) -> Self {
        LoxType::Callable(Rc::new(RefCell::new(value)))
    }
}
impl From<LoxInstance> for LoxType {
    fn from(value: LoxInstance) -> Self {
        LoxType::Data(Rc::new(RefCell::new(value)))
    }
}
