use crate::{
    gen_struct,
    tokens::token::{LiteralType, Token},
};

use super::expr::{Expr, Literal};

gen_struct!(Stmt,
    Expression, expression:Expr;
    Print, expression:Expr;
    Var, name: Token, initializer: Expr;
    Block, statements: Vec<Stmt>
);

impl Default for Stmt {
    fn default() -> Self {
        return Stmt::Expression(Expression::new(Expr::Literal(Literal::new(
            LiteralType::None,
        ))));
    }
}
