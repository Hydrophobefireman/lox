use crate::{
    gen_struct,
    tokens::token::{LoxType, Token},
};

gen_struct!(Expr,
    Binary, left: Box<Expr>, operator: Token, right: Box<Expr>, depth:Option<i32>;
    Call, callee: Box<Expr>, paren: Token,args: Vec<Expr>,depth:Option<i32>;
    Get, object: Box<Expr>, name: Token, depth: Option<i32>;
    Set, object: Box<Expr>, name: Token, value: Box<Expr>, depth: Option<i32>;
    Super, keyword: Token, method: Token, depth: Option<i32>;
    This, keyword: Token, depth: Option<i32>;
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
    pub fn set_depth(&mut self, dpth: i32) {
        let dpth  = Some(dpth);
        match self {
            Expr::Assign(x) => x.depth = dpth,
            Expr::Binary(x) => x.depth = dpth,
            Expr::Call(x) => x.depth = dpth,
            Expr::Grouping(x) => x.depth = dpth,
            Expr::Literal(x) => x.depth = dpth,
            Expr::Logical(x) => x.depth = dpth,
            Expr::Unary(x) => x.depth = dpth,
            Expr::Variable(x) => x.depth = dpth,
            Expr::Get(x) => x.depth = dpth,
            Expr::Set(x) => x.depth = dpth,
            Expr::This(x) => x.depth = dpth,
            Expr::Super(x) => x.depth = dpth,
        };
    }
    pub fn get_depth(&self) -> Option<i32> {
        match self {
            Expr::Assign(x) => x.depth,
            Expr::Binary(x) => x.depth,
            Expr::Call(x) => x.depth,
            Expr::Grouping(x) => x.depth,
            Expr::Literal(x) => x.depth,
            Expr::Logical(x) => x.depth,
            Expr::Unary(x) => x.depth,
            Expr::Variable(x) => x.depth,
            Expr::Get(x) => x.depth,
            Expr::Set(x) => x.depth,
            Expr::This(x) => x.depth,
            Expr::Super(x) => x.depth,
        }
    }
}
