use crate::{
    gen_struct,
    tokens::token::{LiteralType, Token},
};

use super::expr::{Expr, Literal};

gen_struct!(Stmt,
    Expression, expression:Expr;
    If, cond: Expr,then_branch:Box<Stmt>, else_branch:Option<Box<Stmt>>;
    Print, expression:Expr;
    Var, name: Token, initializer: Expr;
    While, cond: Expr, body: Box<Stmt>;
    Block, statements: Vec<Stmt>
);

impl Default for Stmt {
    fn default() -> Self {
        return Stmt::Expression(Expression::new(Expr::Literal(Literal::new(
            LiteralType::InternalNoValue,
        ))));
    }
}
