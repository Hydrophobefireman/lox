use crate::{
    expr::{Binary, Expr},
    tokens::token::Token,
};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}
macro_rules! check {
    ($p:expr,  $(|)? $( $pattern:pat_param )|+) => {
        match $p {
            Some(a) => matches!(a.ty,  $( $pattern )|+),
            None => false, /* ?? */
        }
    };
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Parser { current: 0, tokens }
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        use crate::tokens::token_type::TokenType::{Bang, BangEqual};
        let mut expr = self.comparision();
        while check!(self.peek(), Bang | BangEqual) {
            let operator = (*self.advance()).clone();
            let right = self.comparision();
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)))
        }
        expr
    }

    fn comparision(&mut self) -> Expr {
        use crate::tokens::token_type::TokenType::{Greater, GreaterEqual, Less, LessEqual};

        let mut expr = self.term();

        while check!(self.peek(), Greater | GreaterEqual | Less | LessEqual) {
            let operator = (*self.advance()).clone();
            let right = self.term();
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        expr
    }
    fn term(&mut self) -> Expr {
        Default::default()
    }

    #[inline]
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    #[inline]
    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }
    #[inline]
    fn is_at_end(&self) -> bool {
        check!(self.peek(), crate::tokens::token_type::TokenType::EOF)
    }

    #[inline]
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        };
        self.previous().unwrap()
    }
}
