use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    errors::RuntimeResult,
    interpreter::Interpreter,
    tokens::token::{LoxCallable, LoxCollableType, LoxType},
};
#[derive(Debug)]
pub struct Clock;

impl LoxCallable for Clock {
    fn kind(&self) -> LoxCollableType {
        LoxCollableType::NativeFunction
    }
    fn name(&self) -> String {
        "Clock".into()
    }

    fn arity(&self) -> usize {
        0
    }

    fn call(&mut self, _: &mut Interpreter, _: Vec<LoxType>) -> RuntimeResult<LoxType> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Ok(since_the_epoch.as_secs_f64().into())
    }
    fn clone_box(&self) -> Box<dyn LoxCallable> {
        Box::new(Self {})
    }
}

impl From<Clock> for LoxType {
    fn from(value: Clock) -> Self {
        LoxType::Callable(Box::new(value))
    }
}
