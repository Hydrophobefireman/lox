use crate::{
    expr::{Expr, Visitor},
    program::Program,
    tokens::token_type::TokenType,
};

pub struct Interpreter<'a> {
    program: &'a Program,
}

impl<'a> Interpreter<'a> {
    #[inline]
    fn evaluate(&self, e: &Expr) -> Box<dyn std::any::Any> {
        e.accept(self)
    }

    #[inline]
    fn is_truthy(&self, e: &dyn std::any::Any) -> bool {
        match e.downcast_ref::<bool>() {
            Some(val) => *val,
            None => match e.downcast_ref::<Option<i32>>() {
                Some(v) => v.is_some(),
                None => true,
            },
        }
    }
    #[inline]
    fn cast<F: FnOnce() -> f64>(&self, x: Box<dyn std::any::Any>, or_else: F) -> f64 {
        match x.downcast_ref::<f64>() {
            None => or_else(),
            Some(val) => *val,
        }
    }

    pub fn stringify(&self, e: Box<dyn std::any::Any>) -> String {
        match e.downcast_ref::<f64>() {
            Some(v) => v.to_string(),
            None => match e.downcast_ref::<String>() {
                Some(v) => v.to_owned(),
                None => match e.downcast_ref::<bool>() {
                    Some(val) => val.to_string(),
                    None => match e.downcast_ref::<Option<i32>>() {
                        Some(v) => {
                            if v.is_none() {
                                "Nil".into()
                            } else {
                                "??".into()
                            }
                        }
                        None => "???".into(),
                    },
                },
            },
        }
    }
    pub fn interpret(&self, e: &Expr) {
        let val = self.evaluate(e);
        dbg!(self.stringify(val));
    }

    pub fn new(program: &'a Program) -> Self {
        Self { program }
    }

    fn is_equal(&self, left: &dyn std::any::Any, right: &dyn std::any::Any) -> bool {
        match left.downcast_ref::<bool>() {
            Some(val) => right.downcast_ref::<bool>().unwrap_or(&!*val) == val,
            None => match left.downcast_ref::<Option<i32>>() {
                Some(v) => v.is_some() && right.downcast_ref::<Option<i32>>().is_some(),
                None => match left.downcast_ref::<String>() {
                    None => false,
                    Some(st) => match right.downcast_ref::<String>() {
                        None => false,
                        Some(r_str) => st == r_str,
                    },
                },
            },
        }
    }
}

impl<'a> Visitor<Box<dyn std::any::Any>> for Interpreter<'a> {
    fn Binary(&self, e: &crate::expr::Binary) -> Box<dyn std::any::Any> {
        let left = self.evaluate(&e.left);
        let right = self.evaluate(&e.right);
        let or_else = || {
            self.program
                .error(0, "Cannot perform action on non number value");
            return 0_f64;
        };

        match e.operator.ty {
            TokenType::Minus => Box::new(self.cast(left, or_else) - self.cast(right, or_else)),

            TokenType::Slash => Box::new(self.cast(left, or_else) / self.cast(right, or_else)),
            TokenType::Star => Box::new(self.cast(left, or_else) * self.cast(right, or_else)),

            TokenType::Plus => {
                if left.downcast_ref::<f64>().is_some() && right.downcast_ref::<f64>().is_some() {
                    Box::new(self.cast(left, or_else) + self.cast(right, or_else))
                } else if left.downcast_ref::<String>().is_some()
                    && right.downcast_ref::<String>().is_some()
                {
                    let mut res = left.downcast_ref::<String>().unwrap().clone();
                    res.push_str(right.downcast_ref::<String>().unwrap());
                    Box::new(res)
                } else {
                    self.program.error(0, "Unknown values being added!");
                    Box::new(None::<i32>)
                }
            }
            TokenType::Greater => Box::new(self.cast(left, or_else) > self.cast(right, or_else)),
            TokenType::GreaterEqual => {
                Box::new(self.cast(left, or_else) >= self.cast(right, or_else))
            }
            TokenType::Less => Box::new(self.cast(left, or_else) < self.cast(right, or_else)),
            TokenType::LessEqual => Box::new(self.cast(left, or_else) <= self.cast(right, or_else)),

            TokenType::BangEqual => Box::new(!self.is_equal(&left, &right)),
            TokenType::EqualEqual => Box::new(self.is_equal(&left, &right)),
            _ => panic!("?"),
        }
    }

    #[inline]
    fn Grouping(&self, e: &crate::expr::Grouping) -> Box<dyn std::any::Any> {
        self.evaluate(&e.expression)
    }

    #[inline]
    fn Literal(&self, e: &crate::expr::Literal) -> Box<dyn std::any::Any> {
        match &e.value {
            crate::tokens::token::LiteralType::String(s) => Box::new(s.clone()),
            crate::tokens::token::LiteralType::Float(num) => Box::new(*num),
            crate::tokens::token::LiteralType::True => Box::new(true),
            crate::tokens::token::LiteralType::False => Box::new(false),
            crate::tokens::token::LiteralType::Nil => Box::new(None::<i32>),
            crate::tokens::token::LiteralType::None => panic!("unreachable"),
        }
    }

    fn Unary(&self, e: &crate::expr::Unary) -> Box<dyn std::any::Any> {
        let right = self.evaluate(&e.right);
        match e.operator.ty {
            TokenType::Plus => {
                self.program
                    .report(0, "Runtime", "+{value} is not supported!");
                Box::new(0_f64)
            }
            TokenType::Minus => match right.downcast_ref::<f64>() {
                None => {
                    self.program
                        .report(0, "Invalid syntax", "- not supported here");
                    Box::new(0_f64)
                }
                Some(val) => Box::new(-*val),
            },
            TokenType::Bang => Box::new(!self.is_truthy(&right)),
            _ => panic!("?"),
        }
    }
}
