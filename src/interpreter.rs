use std::{cell::RefCell, rc::Rc};

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
        token::{literal_to_float, LoxType},
        token_type::TokenType,
    },
};

pub struct Interpreter {
    env: EnclosingEnv,
    pub globals: EnclosingEnv,
}

impl Interpreter {
    fn evaluate(&mut self, e: &Expr) -> RuntimeResult<LoxType> {
        e.accept(self)
    }

    #[inline]
    fn is_truthy(&self, e: &LoxType) -> bool {
        !matches!(e, LoxType::False)
    }
    #[inline]
    pub fn stringify(&self, e: &LoxType) -> String {
        e.to_string()
    }
    #[inline]
    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> RuntimeResult<LoxType> {
        let mut result = Default::default();
        for stmt in statements {
            result = self.execute(stmt)?;
        }
        Ok(result)
    }
    #[inline]
    fn execute(&mut self, stmt: &Stmt) -> RuntimeResult<LoxType> {
        Ok(stmt.accept(self)?)
    }
    #[inline]
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        globals.borrow_mut().define("clock", (Clock {}).into());
        Self {
            env: Rc::clone(&globals),
            globals,
        }
    }
    fn is_equal(&self, left: &LoxType, right: &LoxType) -> bool {
        match [left, right] {
            [LoxType::String(l), LoxType::String(r)] => *l == *r,
            [LoxType::Float(l), LoxType::Float(r)] => *l == *r,
            [LoxType::Nil, LoxType::Nil]
            | [LoxType::True, LoxType::True]
            | [LoxType::False, LoxType::False]
            | [LoxType::InternalNoValue, LoxType::InternalNoValue] => true,
            _ => false,
        }
    }

    pub fn execute_block(&mut self, statements: &Vec<Stmt>, env: Environment) -> RuntimeResult<()> {
        let previous = Rc::clone(&self.env);

        self.env = Rc::new(RefCell::new(env));
        for stmt in statements {
            match self.execute(stmt) {
                Err(e) => {
                    self.env = Rc::clone(&previous);
                    Err(e)?
                }
                _ => (),
            };
        }
        self.env = previous;
        Ok(())
    }
}

impl expr::Visitor<RuntimeResult<LoxType>> for Interpreter {
    fn Binary(&mut self, e: &Binary) -> RuntimeResult<LoxType> {
        let left = self.evaluate(&*e.left.clone())?;
        let right = self.evaluate(&*e.right)?;
        match e.operator.ty {
            TokenType::Minus => Ok(LoxType::Float(
                literal_to_float(left)? - literal_to_float(right)?,
            )),

            TokenType::Slash => Ok(LoxType::Float(
                literal_to_float(left)? / literal_to_float(right)?,
            )),
            TokenType::Star => Ok(LoxType::Float(
                literal_to_float(left)? * literal_to_float(right)?,
            )),

            TokenType::Plus => match [&left, &right] {
                [LoxType::String(left_str), LoxType::String(right_str)] => {
                    Ok(LoxType::String(left_str.clone() + right_str))
                }

                [LoxType::Float(l), LoxType::Float(r)] => Ok(LoxType::Float(l + r)),
                [a, b] => Err(RuntimeError::new(
                    &format!(
                        "Invalid addition. Operands must be 2 strings or 2 numbers. Found: {a}, {b}"
                    ),
                    e.operator.line,
                )),
            },
            TokenType::Greater => Ok(LoxType::from(
                literal_to_float(left)? > literal_to_float(right)?,
            )),
            TokenType::GreaterEqual => Ok(LoxType::from(
                literal_to_float(left)? >= literal_to_float(right)?,
            )),
            TokenType::Less => Ok(LoxType::from(
                literal_to_float(left)? < literal_to_float(right)?,
            )),
            TokenType::LessEqual => Ok(LoxType::from(
                literal_to_float(left)? <= literal_to_float(right)?,
            )),

            TokenType::BangEqual => Ok(LoxType::from(!self.is_equal(&left, &right))),
            TokenType::EqualEqual => Ok(LoxType::from(self.is_equal(&left, &right))),
            _ => panic!("?"),
        }
        .map_err(|err| RuntimeError::new(&err.message, e.operator.line))
    }

    #[inline]
    fn Grouping(&mut self, e: &Grouping) -> RuntimeResult<LoxType> {
        self.evaluate(&*e.expression)
    }

    #[inline]
    fn Literal(&mut self, e: &Literal) -> RuntimeResult<LoxType> {
        Ok(e.value.clone())
    }

    fn Unary(&mut self, e: &Unary) -> RuntimeResult<LoxType> {
        let right = self.evaluate(&*e.right)?;
        match e.operator.ty {
            TokenType::Plus => Err(RuntimeError::new(
                "+{value} is not supported",
                e.operator.line,
            )),
            TokenType::Minus => match right {
                LoxType::Float(f) => Ok(LoxType::Float(-f)),
                _ => Err(RuntimeError::new(
                    "Cannot perform negation on non number",
                    e.operator.line,
                )),
            },
            TokenType::Bang => Ok(LoxType::from(!self.is_truthy(&right))),
            _ => panic!("?"),
        }
    }

    fn Variable(&mut self, e: &expr::Variable) -> RuntimeResult<LoxType> {
        Ok(self.env.borrow().get(&e.name)?.clone())
    }
    fn Assign(&mut self, e: &expr::Assign) -> RuntimeResult<LoxType> {
        let val = self.evaluate(&*e.value)?;
        self.env.borrow_mut().assign(e.name.clone(), val.clone())?;
        Ok(val)
    }

    fn Logical(&mut self, e: &expr::Logical) -> RuntimeResult<LoxType> {
        let left = self.evaluate(&*e.left)?;
        match e.operator.ty {
            TokenType::Or => {
                if self.is_truthy(&left) {
                    return Ok(left);
                }
            }
            TokenType::And => {
                if !self.is_truthy(&left) {
                    return Ok(left);
                }
            }
            _ => unreachable!(),
        }
        self.evaluate(&*e.right)
    }
    fn Call(&mut self, e: &expr::Call) -> RuntimeResult<LoxType> {
        let mut callee = self.evaluate(&*e.callee)?;
        let args: Result<Vec<_>, _> = e.args.iter().map(|arg| self.evaluate(arg)).collect();
        let args = args?;

        match &mut callee {
            LoxType::Callable(f) => {
                if args.len() != f.arity() {
                    return Err(RuntimeError::new(
                        &format!("Expected {} args, got {}", f.arity(), args.len()),
                        e.paren.line,
                    ));
                }
                f.call(self, args)
            }
            _ => return Err(RuntimeError::new("Cannot call uncallable", e.paren.line)),
        }
    }
}

impl stmt::Visitor<RuntimeResult<LoxType>> for Interpreter {
    fn Expression(&mut self, e: &stmt::Expression) -> RuntimeResult<LoxType> {
        self.evaluate(&e.expression)
    }

    fn Print(&mut self, e: &stmt::Print) -> RuntimeResult<LoxType> {
        let ev = self.evaluate(&e.expression)?;
        let value = self.stringify(&ev);
        println!("{value}");
        Ok(LoxType::InternalNoValue)
    }

    fn Var(&mut self, e: &stmt::Var) -> RuntimeResult<LoxType> {
        let value = self.evaluate(&e.initializer)?;
        self.env.borrow_mut().define(&e.name.lexeme, value);
        Ok(LoxType::InternalNoValue)
    }
    fn Block(&mut self, e: &stmt::Block) -> RuntimeResult<LoxType> {
        self.execute_block(&e.statements, Environment::new(Some(Rc::clone(&self.env))))?;
        Ok(LoxType::Nil)
    }
    fn If(&mut self, e: &stmt::If) -> RuntimeResult<LoxType> {
        let val = self.evaluate(&e.cond)?;
        if self.is_truthy(&val) {
            self.execute(&*e.then_branch)?;
        } else {
            if let Some(val) = &e.else_branch {
                self.execute(&*val)?;
            };
        }
        Ok(LoxType::InternalNoValue)
    }

    fn While(&mut self, e: &stmt::While) -> RuntimeResult<LoxType> {
        loop {
            let value = self.evaluate(&e.cond)?;
            if !self.is_truthy(&value) {
                break;
            }
            self.execute(&*e.body)?;
        }
        Ok(LoxType::InternalNoValue)
    }
    fn Function(&mut self, e: &stmt::Function) -> RuntimeResult<LoxType> {
        let function = LoxFunction::new(e.clone());
        self.env
            .borrow_mut()
            .define(&e.name.lexeme, function.into());
        Ok(Default::default())
    }
}
