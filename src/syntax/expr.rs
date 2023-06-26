use crate::{
    gen_struct,
    tokens::token::{LoxType, Token},
};

gen_struct!(Expr,
    Binary, left: Box<Expr>, operator: Token, right: Box<Expr>;
    Call, callee: Box<Expr>, paren: Token,args: Vec<Expr>;
    Grouping, expression: Box<Expr>;
    Literal, value: LoxType;
    Logical, left: Box<Expr>, operator: Token, right: Box<Expr>;
    Unary, operator: Token, right: Box<Expr>;
    Variable, name: Token;
    Assign, name: Token, value: Box<Expr>
);

impl Default for Expr {
    #[inline]
    fn default() -> Self {
        Expr::Literal(Literal {
            value: LoxType::InternalNoValue,
        })
    }
}
impl From<LoxType> for Literal {
    #[inline]
    fn from(value: LoxType) -> Self {
        Self::new(value)
    }
}

impl From<LoxType> for Expr {
    #[inline]
    fn from(value: LoxType) -> Self {
        let l: Literal = value.into();
        l.into()
    }
}
