use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{LoxClassError, LoxClassResult, RuntimeResult},
    interpreter::Interpreter,
    lox_function::LoxFunction,
    tokens::token::{ref_cell, LoxCallable, LoxCallableType, LoxInstanceValue, LoxType, Token},
};

pub type SuperClass = Option<Rc<RefCell<dyn LoxCallable>>>;

#[derive(Debug, Clone)]
pub struct LoxClass {
    name: String,
    methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
    superclass: SuperClass,
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
    pub this: LoxClass,
    native_fields: HashMap<String, Rc<RefCell<dyn Any>>>,
    fields: HashMap<String, LoxType>,
}

impl LoxClass {
    pub fn new<T: Into<String>>(
        name: T,
        methods: HashMap<String, Rc<RefCell<LoxFunction>>>,
        superclass: SuperClass,
    ) -> Self {
        Self {
            name: name.into(),
            methods,
            superclass,
        }
    }
    pub fn find_method<T: Into<String> + Clone>(
        &self,
        method: T,
    ) -> Option<Rc<RefCell<LoxFunction>>> {
        let method_ = method.clone();

        match self.methods.get(&(method).into()) {
            Some(x) => Some(Rc::clone(x)),
            None => match &self.superclass {
                Some(sc) => {
                    let val = sc
                        .borrow()
                        .constructor()
                        .expect("Cannot have non constructor here")
                        .find_method(method_.into());
                    val.map(|x| Rc::clone(&x))
                }
                None => None,
            },
        }
    }
}

impl LoxInstance {
    pub fn new(this: LoxClass) -> Self {
        Self {
            this,
            fields: Default::default(),
            native_fields: Default::default(),
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
    pub fn store_native<T: Into<String>>(&mut self, key: T, val: Rc<RefCell<dyn Any>>) -> () {
        self.native_fields.insert(key.into(), val);
    }
    pub fn get_native(&mut self, key: &str) -> Option<&Rc<RefCell<dyn Any>>> {
        self.native_fields.get(key)
    }
}

impl LoxCallable for LoxClass {
    fn constructor(&self) -> Option<&LoxClass> {
        Some(self)
    }
    fn kind(&self) -> LoxCallableType {
        LoxCallableType::Class
    }
    fn arity(&self) -> usize {
        match self.find_method("init") {
            None => 0,
            Some(init) => init.borrow().arity(),
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
            let (res, interp) = initializer
                .borrow()
                .bind(inst.into())
                .call(interpreter, args)?;
            return Ok((res, interp));
        }
        Ok((inst.into(), interpreter))
    }
}

impl From<LoxClass> for LoxType {
    fn from(value: LoxClass) -> Self {
        LoxType::Callable(ref_cell((value)))
    }
}
impl From<LoxInstance> for LoxType {
    fn from(value: LoxInstance) -> Self {
        LoxType::Data(ref_cell((value)))
    }
}
