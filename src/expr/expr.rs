use crate::{
    generate_expr_struct,
    tokens::token::{LiteralType, Token},
};

generate_expr_struct!(
    Binary, left: Box<Expr>, operator: Token, right: Box<Expr>;
    Grouping, expression: Box<Expr>;
    Literal, value: LiteralType;
    Unary, operator: Token, right: Box<Expr>
);
impl Default for Expr {
    fn default() -> Self {
        Expr::Literal(Literal {
            value: LiteralType::Float(0_f64),
        })
    }
}
