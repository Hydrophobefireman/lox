use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{RuntimeError, RuntimeResult},
    tokens::token::{LiteralType, Token},
};

pub type EnclosingEnv = Rc<RefCell<Environment>>;
#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, LiteralType>,
    enclosing: Option<EnclosingEnv>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }
}

impl Environment {
    #[inline]
    pub fn new(enclosing: Option<EnclosingEnv>) -> Self {
        Self {
            values: Default::default(),
            enclosing,
        }
    }
    #[inline]
    pub fn define(&mut self, name: String, value: LiteralType) {
        self.values.insert(name.into(), value);
    }

    pub fn assign(&mut self, name: Token, value: LiteralType) -> RuntimeResult<()> {
        if self.values.contains_key(&name.lexeme) {
            self.define(name.lexeme, value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(outer) => {
                    outer.borrow_mut().assign(name, value)?;
                    Ok(())
                }
                None => Err(RuntimeError::new(
                    &format!("Undefined variable '{}'", name.lexeme),
                    name.line,
                )),
            }
        }
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<LiteralType> {
        if self.values.contains_key(&name.lexeme) {
            self.values
                .get(&name.lexeme)
                .ok_or_else(|| {
                    RuntimeError::new(&format!("Undefined variable '{}'", name.lexeme), name.line)
                })
                .map(|f| f.clone())
        } else {
            match &self.enclosing {
                Some(outer) => outer.borrow().get(name),
                None => Err(RuntimeError::new(
                    &format!("Undefined variable '{}'", name.lexeme),
                    name.line,
                )),
            }
        }
    }
}
