use crate::{
    gen_struct,
    tokens::token::{LiteralType, Token},
};

gen_struct!(Expr,
    Binary, left: Box<Expr>, operator: Token, right: Box<Expr>;
    Grouping, expression: Box<Expr>;
    Literal, value: LiteralType;
    Unary, operator: Token, right: Box<Expr>;
    Variable, name: Token;
    Assign, name: Token, value: Box<Expr>
);

impl Default for Expr {
    #[inline]
    fn default() -> Self {
        Expr::Literal(Literal {
            value: LiteralType::None,
        })
    }
}
impl From<LiteralType> for Literal {
    #[inline]
    fn from(value: LiteralType) -> Self {
        Self::new(value)
    }
}

impl From<LiteralType> for Expr {
    #[inline]
    fn from(value: LiteralType) -> Self {
        let l: Literal = value.into();
        l.into()
    }
}
