use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{RuntimeError, RuntimeResult},
    tokens::token::{LiteralType, Token},
};

type EnclosingEnv = Option<Rc<RefCell<Environment>>>;
pub struct Environment {
    values: HashMap<String, LiteralType>,
    enclosing: EnclosingEnv,
}

impl Environment {
    #[inline]
    pub fn new(enclosing: EnclosingEnv) -> Self {
        Self {
            values: Default::default(),
            enclosing,
        }
    }
    #[inline]
    pub fn define(&mut self, name: String, value: LiteralType) {
        self.values.insert(name.into(), value);
    }

    #[inline]
    pub fn assign(&mut self, name: Token, value: LiteralType) -> RuntimeResult<()> {
        if self.values.contains_key(&name.lexeme) {
            self.define(name.lexeme, value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(outer_scope) => {
                    outer_scope.borrow().assign(name, value)?;
                    Ok(())
                }
                None => Err(RuntimeError::new(
                    &format!("Undefined variable '{}'", name.lexeme),
                    name.line,
                )),
            }
        }
    }

    #[inline]
    pub fn get(&self, name: &Token) -> RuntimeResult<&LiteralType> {
        let res = self.values.get(&name.lexeme);
        match res {
            Some(x) => Ok(x),
            None => match &self.enclosing {
                Some(outer_scope) => outer_scope.borrow().get(name),
                None => Err(RuntimeError::new(
                    &format!("Undefined variable {}", name.lexeme),
                    name.line,
                ))?,
            },
        }
    }
}
