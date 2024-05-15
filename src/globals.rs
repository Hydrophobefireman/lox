use std::{
    cell::RefCell,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    environment::Environment,
    errors::{RuntimeError, RuntimeResult},
    gen_native_func,
    interpreter::{self, Interpreter},
    lox_class::{LoxClass, LoxInstance},
    lox_function::{FunctionKind, LoxFunction},
    syntax::{
        expr::{Call, Expr, Variable},
        stmt::{Function, Return, Stmt},
    },
    tokens::{
        token::{ref_cell, LoxCallable, LoxCallableType, LoxType, Token},
        token_type::TokenType,
    },
};

gen_native_func!(Clock, i, {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs_f64().into()
});

gen_native_func!(
    FileInit,
    interpreter,
    {
        let name = match name {
            LoxType::String(name) => name,
            _ => {
                return Err(RuntimeError::new(
                    "File name must be a string",
                    0,
                    interpreter,
                ))
            }
        };
        let open_type = match open_type {
            LoxType::String(st) => st,
            _ => {
                return Err(RuntimeError::new(
                    "Open type must be a string!",
                    0,
                    interpreter,
                ))
            }
        };
        let mut ot = OpenOptions::new();
        let write_mode = open_type.contains('w');
        let append_mode = open_type.contains('a');
        let f = match ot
            .read(open_type.contains('r'))
            .write(write_mode)
            .append(append_mode)
            .create(write_mode || append_mode)
            .open(name)
        {
            Err(e) => return Err(RuntimeError::new(e.to_string(), 0, interpreter)),
            Ok(f) => f,
        };

        let this = interpreter.env.borrow().get(&Token::dummy_this());
        let this = match this {
            Err(e) => return Err(RuntimeError::new(e.message, e.line, interpreter)),
            Ok(t) => t,
        };
        match &this {
            LoxType::Data(inst) => {
                let mut inst = inst.borrow_mut();
                inst.store_native("file_handle", ref_cell(f));
                inst.store_native("file_options", ref_cell(ot));
            }
            _ => panic!("this refered to an enexpected type!"),
        };

        this
    },
    name,
    open_type
);

gen_native_func!(FileRead, interpreter, {
    let this = interpreter.env.borrow().get(&Token::dummy_this());
    let this = match this {
        Err(e) => return Err(RuntimeError::new(e.message, e.line, interpreter)),
        Ok(t) => t,
    };
    let ret = match &this {
        LoxType::Data(inst) => {
            let mut inst = inst.borrow_mut();
            let file = inst
                .get_native("file_handle")
                .expect("Expected a valid file handle");
            let file = file.borrow();
            let mut f = file
                .downcast_ref::<File>()
                .expect("Expected valid file handle");
            let mut s = String::with_capacity(f.metadata().unwrap().size() as usize);

            match f.read_to_string(&mut s) {
                Err(e) => return Err(RuntimeError::new(e.to_string(), 0, interpreter)),
                Ok(_) => LoxType::String(s),
            }
        }
        _ => panic!("this refered to an enexpected type!"),
    };
    ret
});

gen_native_func!(
    FileWrite,
    interpreter,
    {
        let this = interpreter.env.borrow().get(&Token::dummy_this());
        let this = match this {
            Err(e) => return Err(RuntimeError::new(e.message, e.line, interpreter)),
            Ok(t) => t,
        };
        let text = match text {
            LoxType::String(t) => t,
            _ => {
                return Err(RuntimeError::new(
                    "Invalid type for 'text' for write()",
                    0,
                    interpreter,
                ))
            }
        };
        let ret = match &this {
            LoxType::Data(inst) => {
                let mut inst = inst.borrow_mut();
                let file = inst
                    .get_native("file_handle")
                    .expect("Expected a valid file handle");
                let file = file.borrow();
                let mut f = file
                    .downcast_ref::<File>()
                    .expect("Expected valid file handle");

                match f.write(text.as_bytes()) {
                    Err(e) => return Err(RuntimeError::new(e.to_string(), 0, interpreter)),
                    Ok(_) => Default::default(),
                }
            }
            _ => panic!("this refered to an enexpected type!"),
        };
        ret
    },
    text
);

fn file_cls() -> LoxClass {
    LoxClass::new(
        "File",
        [
            ("init", native_call("init", ref_cell(FileInit))),
            ("read", native_call("read", ref_cell(FileRead))),
            ("write", native_call("write", ref_cell(FileWrite))),
        ]
        .map(|(x, y)| (x.to_owned(), ref_cell(y)))
        .into(),
        None,
    )
}

fn io() -> LoxInstance {
    let cls = LoxClass::new("io", Default::default(), None);
    LoxInstance::new(cls)
}

pub fn initialize_globals() -> Environment {
    let mut env = Environment::new(None);
    env.define("clock", Clock {}.into());
    env.define("File", file_cls().into());
    env
}

/// create a wrapper function that simply calls a native function
pub fn native_call<T: Into<String>>(name: T, f: Rc<RefCell<dyn LoxCallable>>) -> LoxFunction {
    let args = f.borrow().arity();
    let fun_token = Token::dummy(name, TokenType::String);
    let params = (0..args).map(|id| Token::dummy(format!("_{id}"), TokenType::String));
    let call_args = (0..args).map(|id| {
        Expr::Variable(Variable::new(
            Token::dummy(format!("_{id}"), TokenType::String),
            Some(0),
        ))
    });

    let call_stmt = Stmt::Return(Return::new(
        Token::dummy("return", TokenType::Return),
        Some(
            Call::new(
                Box::new(Variable::new(fun_token.clone(), Some(2)).into()),
                Token::dummy("(", TokenType::LeftParen),
                call_args.collect(),
                Some(0),
            )
            .into(),
        ),
    ));

    let mut env = Environment::new(None);
    env.define(fun_token.lexeme.clone(), LoxType::Callable(f));
    LoxFunction::new(
        Function::new(fun_token, params.collect(), vec![call_stmt]),
        ref_cell(env),
        FunctionKind::Function,
    )
}
