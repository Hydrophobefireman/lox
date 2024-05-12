use crate::{
    gen_struct,
    tokens::token::{LoxType, Token},
};

use super::expr::{Expr, Literal};

gen_struct!(Stmt,
    Expression, expression:Expr;
    Function, name: Token, params: Vec<Token>, body:Vec<Stmt>;
    If, cond: Expr,then_branch:Box<Stmt>, else_branch:Option<Box<Stmt>>;
    Print, expression:Expr;
    Return, keyword: Token, value: Option<Expr>;
    Var, name: Token, initializer: Expr;
    While, cond: Expr, body: Box<Stmt>;
    Block, statements: Vec<Stmt>
);

impl Default for Stmt {
    fn default() -> Self {
        return Stmt::Expression(Expression::new(Expr::Literal(Literal::new(
            LoxType::InternalNoValue,
            None,
        ))));
    }
}
