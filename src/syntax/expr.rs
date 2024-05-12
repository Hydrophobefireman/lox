use crate::{
    gen_struct,
    tokens::token::{LoxType, Token},
};

gen_struct!(Expr,
    Binary, left: Box<Expr>, operator: Token, right: Box<Expr>, depth:Option<i32>;
    Call, callee: Box<Expr>, paren: Token,args: Vec<Expr>,depth:Option<i32>;
    Grouping, expression: Box<Expr>,depth:Option<i32>;
    Literal, value: LoxType,depth:Option<i32>;
    Logical, left: Box<Expr>, operator: Token, right: Box<Expr>,depth:Option<i32>;
    Unary, operator: Token, right: Box<Expr>,depth:Option<i32>;
    Variable, name: Token,depth:Option<i32>;
    Assign, name: Token, value: Box<Expr>,depth:Option<i32>
);

impl Default for Expr {
    #[inline]
    fn default() -> Self {
        Expr::Literal(Literal {
            value: LoxType::InternalNoValue,
            depth: None,
        })
    }
}
impl From<LoxType> for Literal {
    #[inline]
    fn from(value: LoxType) -> Self {
        Self::new(value, None)
    }
}

impl From<LoxType> for Expr {
    #[inline]
    fn from(value: LoxType) -> Self {
        let l: Literal = value.into();
        l.into()
    }
}

impl Expr {
    pub fn depth(&mut self, dpth: i32) {
        match self {
            Expr::Assign(x) => {
                x.depth = Some(dpth);
            }
            Expr::Binary(x) => {
                x.depth = Some(dpth);
            }
            Expr::Call(x) => {
                x.depth = Some(dpth);
            }
            Expr::Grouping(x) => {
                x.depth = Some(dpth);
            }
            Expr::Literal(x) => {
                x.depth = Some(dpth);
            }
            Expr::Logical(x) => {
                x.depth = Some(dpth);
            }
            Expr::Unary(x) => {
                x.depth = Some(dpth);
            }
            Expr::Variable(x) => {
                x.depth = Some(dpth);
            }
        };
    }
}
