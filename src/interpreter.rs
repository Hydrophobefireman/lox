use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    environment::{EnclosingEnv, Environment},
    errors::{RuntimeError, RuntimeResult},
    globals::Clock,
    lox_function::LoxFunction,
    syntax::{
        expr::{self, Binary, Expr, Grouping, Literal, Unary},
        stmt::{self, Stmt},
    },
    tokens::{
        token::{LoxType, Token},
        token_type::TokenType,
    },
};

#[derive(Debug)]
pub struct Interpreter {
    env: EnclosingEnv,
    locals: HashMap<Expr, usize>,
    pub globals: EnclosingEnv,
}

impl Interpreter {
    fn is_truthy(&self, e: &LoxType) -> bool {
        !matches!(e, LoxType::False)
    }

    pub fn stringify(&self, e: &LoxType) -> String {
        e.to_string()
    }

    pub fn interpret(self, statements: Vec<Stmt>) -> RuntimeResult<(LoxType, Self)> {
        let mut result = Default::default();
        let mut this = self;
        for stmt in statements {
            (result, this) = this.execute(stmt)?;
        }
        Ok((result, this))
    }
    fn evaluate(self, expr: Expr) -> RuntimeResult<(LoxType, Self)> {
        match expr {
            Expr::Assign(e) => {
                let (val, this) = self.evaluate(*e.value)?;
                let put = this.env.borrow_mut().assign(e.name.clone(), val.clone());
                match put {
                    Err(e) => return Err(RuntimeError::new(e.message, e.line, this)),
                    _ => (),
                }
                Ok((val, this))
            }

            Expr::Binary(e) => {
                let (left, this) = self.evaluate(*e.left)?;
                let (right, this) = this.evaluate(*e.right)?;

                fn float_op(a: LoxType, b: LoxType, op: TokenType) -> Result<LoxType, ()> {
                    if let LoxType::Float(a) = a {
                        if let LoxType::Float(b) = b {
                            let lv: LoxType = match op {
                                TokenType::Minus => (a - b).into(),
                                TokenType::Slash => (a / b).into(),
                                TokenType::Star => (a * b).into(),
                                TokenType::Plus => (a + b).into(),
                                TokenType::Greater => (a > b).into(),
                                TokenType::GreaterEqual => (a >= b).into(),
                                TokenType::Less => (a < b).into(),
                                TokenType::LessEqual => (a <= b).into(),
                                TokenType::BangEqual => (a != b).into(),
                                TokenType::EqualEqual => (a == b).into(),
                                _ => panic!("Unknown binary op!"),
                            };
                            return Ok(lv);
                        }
                    }
                    return Err(());
                }
                use TokenType::*;
                match e.operator.ty {
                    ty @ (Minus | Slash | Star | Greater | GreaterEqual | Less | LessEqual
                    | BangEqual | EqualEqual) => {
                        let res = float_op(left, right, ty);
                        match res {
                            Ok(t) => Ok((t, this)),
                            Err(_) => Err({
                                RuntimeError::new(
                                    "Invalid operands for binary operation".into(),
                                    e.operator.line,
                                    this,
                                )
                            }),
                        }
                    }

                    TokenType::Plus => match (left, right) {
                        (LoxType::String(mut left_str), LoxType::String(right_str)) => {
                            left_str.push_str(right_str.as_str());
                            Ok((LoxType::String(left_str), this))
                        }
                        (LoxType::Float(l), LoxType::Float(r)) => Ok(((l + r).into(), this)),

                        (a, b) => {
                            let msg=format!(
                            "Invalid addition. Operands must be 2 strings or 2 numbers. Found: {a}, {b}"
                        );
                            Err(RuntimeError::new(msg, e.operator.line, this))
                        }
                    },
                    _ => panic!("Unknown binary op!"),
                }
            }
            Expr::Call(e) => {
                let (callee, mut this) = self.evaluate(*e.callee)?;
                let mut args = Vec::with_capacity(e.args.len());
                for arg in e.args {
                    let it;
                    (it, this) = this.evaluate(arg)?;
                    args.push(it);
                }

                match callee {
                    LoxType::Callable(mut f) => {
                        if args.len() != f.arity() {
                            return Err(RuntimeError::new(
                                format!("Expected {} args, got {}", f.arity(), args.len()),
                                e.paren.line,
                                this,
                            ));
                        }
                        let (res, this) = f.call(this, args)?;
                        Ok((res, this))
                    }
                    _ => {
                        return Err(RuntimeError::new(
                            "Cannot call uncallable".into(),
                            e.paren.line,
                            this,
                        ))
                    }
                }
            }
            Expr::Grouping(e) => self.evaluate(*e.expression),
            Expr::Literal(e) => Ok((e.value, self)),
            Expr::Logical(e) => {
                let (left, this) = self.evaluate(*e.left)?;
                match e.operator.ty {
                    TokenType::Or => {
                        if this.is_truthy(&left) {
                            return Ok((left, this));
                        }
                    }
                    TokenType::And => {
                        if !this.is_truthy(&left) {
                            return Ok((left, this));
                        }
                    }
                    _ => unreachable!(),
                }
                this.evaluate(*e.right)
            }
            Expr::Unary(e) => {
                let (right, this) = self.evaluate(*e.right)?;
                match e.operator.ty {
                    TokenType::Plus => Err(RuntimeError::new(
                        "+{value} is not supported".into(),
                        e.operator.line,
                        this,
                    )),
                    TokenType::Minus => match right {
                        LoxType::Float(f) => Ok((LoxType::Float(-f), this)),
                        _ => Err(RuntimeError::new(
                            "Cannot perform negation on non number".into(),
                            e.operator.line,
                            this,
                        )),
                    },
                    TokenType::Bang => Ok(((!this.is_truthy(&right)).into(), this)),
                    _ => panic!("?"),
                }
            }
            Expr::Variable(e) => {
                let val = self.env.borrow().get(&e.name);
                match val {
                    Err(e) => Err(RuntimeError::new(e.message, e.line, self)),
                    Ok(val) => Ok((val, self)),
                }
            }
        }
    }
    fn execute(self, stmt: Stmt) -> RuntimeResult<(LoxType, Self)> {
        match stmt {
            Stmt::Block(e) => {
                let new_env = Environment::new(Some(Rc::clone(&self.env)));
                let this = self.execute_block(e.statements, new_env)?;
                Ok((LoxType::Nil, this))
            }
            Stmt::Expression(e) => self.evaluate(e.expression),
            Stmt::Function(e) => {
                let function = LoxFunction::new(e.clone(), Rc::clone(&self.env));
                self.env
                    .borrow_mut()
                    .define(&e.name.lexeme, function.into());
                Ok((Default::default(), self))
            }
            Stmt::If(e) => {
                let cond = e.cond;
                let branch = e.then_branch;
                let else_branch = e.else_branch;
                let (val, mut this) = self.evaluate(cond)?;
                if this.is_truthy(&val) {
                    (_, this) = this.execute(*branch)?;
                } else if let Some(val) = else_branch {
                    (_, this) = this.execute(*val)?;
                }
                Ok((Default::default(), this))
            }
            Stmt::Print(e) => {
                let (ev, this) = self.evaluate(e.expression)?;
                let value = this.stringify(&ev);
                println!("{value}");
                Ok((LoxType::InternalNoValue, this))
            }
            Stmt::Return(e) => {
                let (value, this) = match e.value {
                    Some(val) => self.evaluate(val)?,
                    None => (LoxType::Nil, self),
                };
                Err(RuntimeError::as_return(value, this))
            }
            Stmt::Var(e) => {
                let (value, this) = self.evaluate(e.initializer)?;
                this.env.borrow_mut().define(&e.name.lexeme, value);
                Ok((LoxType::InternalNoValue, this))
            }
            Stmt::While(e) => {
                let mut this = self;
                let mut value;
                loop {
                    let x = this.evaluate(e.cond.clone())?;
                    (value, this) = x;
                    if !this.is_truthy(&value) {
                        break;
                    }
                    let body = Box::clone(&e.body);
                    (_, this) = this.execute(*body)?;
                }
                Ok((LoxType::InternalNoValue, this))
            }
        }
    }

    pub fn new() -> Self {
        let mut globals = Environment::new(None);
        globals.define("clock", (Clock {}).into());
        let globals = Rc::new(RefCell::new(globals));
        Self {
            env: Rc::clone(&globals),
            globals,
            locals: Default::default(),
        }
    }

    pub fn execute_block(self, statements: Vec<Stmt>, env: Environment) -> RuntimeResult<Self> {
        let mut this = self;
        let previous = Rc::clone(&this.env);

        this.env = Rc::new(RefCell::new(env));
        for stmt in statements {
            match this.execute(stmt) {
                Err(mut e) => {
                    e.interpreter.env = Rc::clone(&previous);
                    return Err(e);
                }
                Ok((_, _this)) => {
                    this = _this;
                }
            };
        }
        this.env = previous;
        Ok(this)
    }
}

// impl expr::Visitor<RuntimeResult<LoxType>> for Interpreter {

//     fn Grouping(&mut self, e: &Grouping) -> RuntimeResult<LoxType> {
//         self.evaluate(&*e.expression)
//     }

//     fn Literal(&mut self, e: &Literal) -> RuntimeResult<LoxType> {
//         Ok(e.value.clone())
//     }

//     fn Unary(&mut self, e: &Unary) -> RuntimeResult<LoxType> {

//     }

//     fn Variable(&mut self, e: &expr::Variable) -> RuntimeResult<LoxType> {
//         Ok(self.env.borrow().get(&e.name)?.clone())
//     }

//     fn Logical(&mut self, e: &expr::Logical) -> RuntimeResult<LoxType> {
//         let left = self.evaluate(&*e.left)?;
//         match e.operator.ty {
//             TokenType::Or => {
//                 if self.is_truthy(&left) {
//                     return Ok(left);
//                 }
//             }
//             TokenType::And => {
//                 if !self.is_truthy(&left) {
//                     return Ok(left);
//                 }
//             }
//             _ => unreachable!(),
//         }
//         self.evaluate(&*e.right)
//     }

// }
