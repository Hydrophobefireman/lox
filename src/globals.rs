use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    errors::RuntimeResult,
    interpreter::Interpreter,
    tokens::token::{LoxCallable, LoxCallableType, LoxType},
};
#[derive(Debug)]
pub struct Clock;

impl LoxCallable for Clock {
    fn kind(&self) -> LoxCallableType {
        LoxCallableType::NativeFunction
    }
    fn name(&self) -> String {
        String::from("Clock")
    }

    fn arity(&self) -> usize {
        0
    }

    fn call(&mut self, i: Interpreter, _: Vec<LoxType>) -> RuntimeResult<(LoxType, Interpreter)> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Ok((since_the_epoch.as_secs_f64().into(), i))
    }
    // fn clone_box(&self) -> Box<dyn LoxCallable> {
    //     Box::new(Self {})
    // }
}

impl From<Clock> for LoxType {
    fn from(value: Clock) -> Self {
        LoxType::Callable(Rc::new(RefCell::new(value)))
    }
}
