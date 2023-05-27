use crate::{
    expr::{Binary, Expr, Grouping, Literal, Unary},
    program::Program,
    tokens::{
        token::{LiteralType, Token},
        token_type::TokenType,
    },
};

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    program: &'a Program,
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
    pub fn new(tokens: &'a Vec<Token>, program: &'a Program) -> Self {
        Parser {
            current: 0,
            tokens,
            program,
        }
    }

    #[inline]
    pub fn parse(&mut self) -> Result<Expr, ()> {
        self.expression()
    }

    #[inline]
    fn expression(&mut self) -> Result<Expr, ()> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ()> {
        use crate::tokens::token_type::TokenType::{Bang, BangEqual};
        let mut expr = self.comparision()?;
        while check!(self.peek(), Bang | BangEqual) {
            let operator = (*self.advance()).clone();
            let right = self.comparision()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)))
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> Result<Expr, ()> {
        use crate::tokens::token_type::TokenType::{Greater, GreaterEqual, Less, LessEqual};

        let mut expr = self.term()?;

        while check!(self.peek(), Greater | GreaterEqual | Less | LessEqual) {
            let operator = (*self.advance()).clone();
            let right = self.term()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        Ok(expr)
    }
    fn term(&mut self) -> Result<Expr, ()> {
        use crate::tokens::token_type::TokenType::{Minus, Plus};
        let mut expr = self.factor()?;

        while check!(self.peek(), Minus | Plus) {
            let operator = (*self.advance()).clone();
            let right = self.factor()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ()> {
        use crate::tokens::token_type::TokenType::{Slash, Star};
        let mut expr = self.unary()?;
        while check!(self.peek(), Slash | Star) {
            let operator = (*self.advance()).clone();
            let right = self.unary()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ()> {
        use crate::tokens::token_type::TokenType::{Bang, Minus};

        if check!(self.peek(), Bang | Minus) {
            let operator = (*self.advance()).clone();
            let right = self.unary()?;
            Ok(Expr::Unary(Unary::new(operator, Box::new(right))))
        } else {
            self.primary()
        }
    }
    fn primary(&mut self) -> Result<Expr, ()> {
        use crate::tokens::token_type::TokenType::{
            False, LeftParen, Nil, Number, RightParen, String, True,
        };
        Ok(match self.advance().ty {
            False => Expr::Literal(Literal::new(LiteralType::False)),
            True => Expr::Literal(Literal::new(LiteralType::True)),
            Nil => Expr::Literal(Literal::new(LiteralType::Nil)),
            Number | String => Expr::Literal(Literal::new(self.previous().literal.clone())),
            LeftParen => {
                let expr = self.expression()?;
                self.consume(RightParen, "Expected ')' after expression")?;
                Expr::Grouping(Grouping::new(Box::new(expr)))
            }
            _ => {
                self.current -= 1; // did not match anything, backtrack
                self.error::<Expr>(self.peek().unwrap(), "Expected expression")?
            }
        })
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().ty == TokenType::Semicolon {
                break;
            }
            match self.peek().unwrap().ty {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => break,
                _ => (),
            }
            self.advance();
        }
    }
    #[inline]
    fn error<T>(&self, t: &Token, err: &str) -> Result<T, ()> {
        match t.ty {
            TokenType::EOF => self.program.error(t.line, &format!(" at the end {err}")),
            _ => self
                .program
                .error(t.line, &format!(" at '{}' {}", t.lexeme, err)),
        };
        Err(())
    }
    #[inline]
    fn consume(&mut self, x: TokenType, err: &str) -> Result<&Token, ()> {
        if self.peek().unwrap().ty == x {
            Ok(self.advance())
        } else {
            let token = self.peek().unwrap();
            self.error(token, err)
        }
    }
    #[inline]
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    #[inline]
    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
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
        self.previous()
    }
}
