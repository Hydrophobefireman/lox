use crate::{
    errors::{RuntimeError, RuntimeResult},
    expr::{Expr, Visitor},
    tokens::{
        token::{literal_to_float, LiteralType},
        token_type::TokenType,
    },
};

pub struct Interpreter {}

impl Interpreter {
    #[inline]
    fn evaluate(&self, e: &Expr) -> RuntimeResult<LiteralType> {
        e.accept(self)
    }

    #[inline]
    fn is_truthy(&self, e: &LiteralType) -> bool {
        !matches!(e, LiteralType::False)
    }
    #[inline]
    pub fn stringify(&self, e: LiteralType) -> String {
        e.to_string()
    }
    pub fn interpret(&self, e: &Expr) -> Result<(), RuntimeError> {
        let val = self.evaluate(e)?;
        dbg!(self.stringify(val));
        Ok(())
    }

    pub fn new() -> Self {
        Self {}
    }

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
}

impl Visitor<RuntimeResult<LiteralType>> for Interpreter {
    fn Binary(&self, e: &crate::expr::Binary) -> RuntimeResult<LiteralType> {
        let left = self.evaluate(&e.left)?;
        let right = self.evaluate(&e.right)?;
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
                    let mut res = left_str.clone();
                    res.push_str(&right_str);
                    Ok(LiteralType::String(res))
                }

                [LiteralType::Float(l), LiteralType::Float(r)] => Ok(LiteralType::Float(l + r)),
                [_, _] => Err(RuntimeError::new("Invalid addition", e.operator.line)),
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
    fn Grouping(&self, e: &crate::expr::Grouping) -> RuntimeResult<LiteralType> {
        self.evaluate(&e.expression)
    }

    #[inline]
    fn Literal(&self, e: &crate::expr::Literal) -> RuntimeResult<LiteralType> {
        Ok(e.value.clone())
    }

    fn Unary(&self, e: &crate::expr::Unary) -> RuntimeResult<LiteralType> {
        let right = self.evaluate(&e.right)?;
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
            TokenType::Bang => Ok(LiteralType::from(!self.is_truthy(&right))),
            _ => panic!("?"),
        }
    }
}
