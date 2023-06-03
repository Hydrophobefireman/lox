use std::{cell::RefCell, rc::Rc};

use crate::{
    environment::{EnclosingEnv, Environment},
    errors::{RuntimeError, RuntimeResult},
    syntax::{
        expr::{self, Binary, Expr, Grouping, Literal, Unary},
        stmt::{self, Stmt},
    },
    tokens::{
        token::{literal_to_float, LiteralType},
        token_type::TokenType,
    },
};

pub struct Interpreter {
    env: EnclosingEnv,
}

impl Interpreter {
    #[inline]
    fn evaluate(&mut self, e: Expr) -> RuntimeResult<LiteralType> {
        e.accept(self)
    }

    #[inline]
    fn is_truthy(&mut self, e: LiteralType) -> bool {
        !matches!(e, LiteralType::False)
    }
    #[inline]
    pub fn stringify(&self, e: &LiteralType) -> String {
        e.to_string()
    }
    #[inline]
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> RuntimeResult<LiteralType> {
        let mut result = Default::default();
        for stmt in statements {
            result = self.execute(stmt)?;
        }
        Ok(result)
    }
    #[inline]
    fn execute(&mut self, stmt: Stmt) -> RuntimeResult<LiteralType> {
        Ok(stmt.accept(self)?)
    }
    #[inline]
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Environment::new(None))),
        }
    }
    #[inline]
    fn is_equal(&self, left: &LiteralType, right: &LiteralType) -> bool {
        match [left, right] {
            [LiteralType::String(l), LiteralType::String(r)] => *l == *r,
            [LiteralType::Float(l), LiteralType::Float(r)] => *l == *r,
            [LiteralType::Nil, LiteralType::Nil]
            | [LiteralType::True, LiteralType::True]
            | [LiteralType::False, LiteralType::False]
            | [LiteralType::None, LiteralType::None] => true,
            _ => false,
        }
    }

    pub fn execute_block(&mut self, statements: Vec<Stmt>, env: Environment) -> RuntimeResult<()> {
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

impl expr::Visitor<RuntimeResult<LiteralType>> for Interpreter {
    fn Binary(&mut self, e: Binary) -> RuntimeResult<LiteralType> {
        let left = self.evaluate(*e.left)?;
        let right = self.evaluate(*e.right)?;
        match e.operator.ty {
            TokenType::Minus => Ok(LiteralType::Float(
                literal_to_float(left)? - literal_to_float(right)?,
            )),

            TokenType::Slash => Ok(LiteralType::Float(
                literal_to_float(left)? / literal_to_float(right)?,
            )),
            TokenType::Star => Ok(LiteralType::Float(
                literal_to_float(left)? * literal_to_float(right)?,
            )),

            TokenType::Plus => match [&left, &right] {
                [LiteralType::String(left_str), LiteralType::String(right_str)] => {
                    Ok(LiteralType::String(left_str.clone() + right_str))
                }

                [LiteralType::Float(l), LiteralType::Float(r)] => Ok(LiteralType::Float(l + r)),
                [a, b] => Err(RuntimeError::new(
                    &format!(
                        "Invalid addition. Operands must be 2 strings or 2 numbers. Found: {a}, {b}"
                    ),
                    e.operator.line,
                )),
            },
            TokenType::Greater => Ok(LiteralType::from(
                literal_to_float(left)? > literal_to_float(right)?,
            )),
            TokenType::GreaterEqual => Ok(LiteralType::from(
                literal_to_float(left)? >= literal_to_float(right)?,
            )),
            TokenType::Less => Ok(LiteralType::from(
                literal_to_float(left)? < literal_to_float(right)?,
            )),
            TokenType::LessEqual => Ok(LiteralType::from(
                literal_to_float(left)? <= literal_to_float(right)?,
            )),

            TokenType::BangEqual => Ok(LiteralType::from(!self.is_equal(&left, &right))),
            TokenType::EqualEqual => Ok(LiteralType::from(self.is_equal(&left, &right))),
            _ => panic!("?"),
        }
        .map_err(|err| RuntimeError::new(&err.message, e.operator.line))
    }

    #[inline]
    fn Grouping(&mut self, e: Grouping) -> RuntimeResult<LiteralType> {
        self.evaluate(*e.expression)
    }

    #[inline]
    fn Literal(&mut self, e: Literal) -> RuntimeResult<LiteralType> {
        Ok(e.value)
    }

    fn Unary(&mut self, e: Unary) -> RuntimeResult<LiteralType> {
        let right = self.evaluate(*e.right)?;
        match e.operator.ty {
            TokenType::Plus => Err(RuntimeError::new(
                "+{value} is not supported",
                e.operator.line,
            )),
            TokenType::Minus => match right {
                LiteralType::Float(f) => Ok(LiteralType::Float(-f)),
                _ => Err(RuntimeError::new(
                    "Cannot perform negation on non number",
                    e.operator.line,
                )),
            },
            TokenType::Bang => Ok(LiteralType::from(!self.is_truthy(right))),
            _ => panic!("?"),
        }
    }

    fn Variable(&mut self, e: expr::Variable) -> RuntimeResult<LiteralType> {
        Ok(self.env.borrow().get(&e.name)?.clone())
    }
    fn Assign(&mut self, e: expr::Assign) -> RuntimeResult<LiteralType> {
        let val = self.evaluate(*e.value)?;
        self.env.borrow_mut().assign(e.name, val.clone())?;
        Ok(val)
    }
}

impl stmt::Visitor<RuntimeResult<LiteralType>> for Interpreter {
    fn Expression(&mut self, e: stmt::Expression) -> RuntimeResult<LiteralType> {
        self.evaluate(e.expression)
    }

    fn Print(&mut self, e: stmt::Print) -> RuntimeResult<LiteralType> {
        let ev = self.evaluate(e.expression)?;
        let value = self.stringify(&ev);
        println!("{value}");
        Ok(LiteralType::None)
    }

    fn Var(&mut self, e: stmt::Var) -> RuntimeResult<LiteralType> {
        let value = self.evaluate(e.initializer)?;
        self.env.borrow_mut().define(e.name.lexeme, value);
        Ok(LiteralType::None)
    }
    fn Block(&mut self, e: stmt::Block) -> RuntimeResult<LiteralType> {
        self.execute_block(e.statements, Environment::new(Some(Rc::clone(&self.env))))?;
        Ok(LiteralType::Nil)
    }
}
