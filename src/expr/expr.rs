use crate::{generate_expr_struct, tokens::token::Token};

generate_expr_struct!(
    Binary, left: Box<Expr>, operator: Token, right: Box<Expr>;
    Grouping, expression: Box<Expr>;
    Literal, value: Option<Box<dyn std::any::Any>>;
    Unary, operator: Token, right: Box<Expr>
);
