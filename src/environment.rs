use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    errors::{EnvError, EnvResult},
    tokens::token::{LoxType, Token},
};

pub type EnclosingEnv = Rc<RefCell<Environment>>;
#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, LoxType>,
    pub enclosing: Option<EnclosingEnv>,
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
    pub fn define<T: Into<String>>(&mut self, name: T, value: LoxType) {
        self.values.insert(name.into(), value);
    }

    pub fn assign(&mut self, name: &Token, value: LoxType) -> EnvResult<()> {
        if self.values.contains_key(&name.lexeme) {
            self.define(&name.lexeme, value);
            Ok(())
        } else {
            match &mut self.enclosing {
                Some(outer) => {
                    outer.borrow_mut().assign(name, value)?;
                    Ok(())
                }
                None => Err(EnvError::new(
                    format!("Undefined variable '{}'", name.lexeme),
                    name.line,
                )),
            }
        }
    }
    pub fn get_at(&self, name: &Token, dist: i32) -> EnvResult<LoxType> {
        if dist == 0 {
            self.values
                .get(&name.lexeme)
                .ok_or_else(|| {
                    EnvError::new(format!("Undefined variable '{}'", name.lexeme), name.line)
                })
                .map(|f| f.clone())
        } else {
            let enc = self.enclosing.as_ref().unwrap();
            let b = enc.borrow();
            let res = b.get_at(name, dist - 1);
            res
        }
    }

    pub fn assign_at(&mut self, name: Token, value: LoxType, dist: i32) -> EnvResult<()> {
        if dist == 0 {
            self.define(&name.lexeme, value);
            Ok(())
        } else {
            let enc = self.enclosing.as_ref().unwrap();
            let mut b = enc.borrow_mut();
            let res = b.assign_at(name, value, dist - 1);
            res
        }
    }

    pub fn get(&self, name: &Token) -> EnvResult<LoxType> {
        if self.values.contains_key(&name.lexeme) {
            self.values
                .get(&name.lexeme)
                .ok_or_else(|| {
                    EnvError::new(format!("Undefined variable '{}'", name.lexeme), name.line)
                })
                .map(|f| f.clone())
        } else {
            match &self.enclosing {
                Some(outer) => outer.borrow().get(name),
                None => Err(EnvError::new(
                    format!("Undefined variable '{}'", name.lexeme),
                    name.line,
                )),
            }
        }
    }
}
