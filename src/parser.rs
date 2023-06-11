use crate::{
    errors::{ParseError, ParseResult},
    syntax::{
        expr::{Assign, Binary, Expr, Grouping, Logical, Unary, Variable},
        stmt::{Block, Expression, If, Print, Stmt, Var, While},
    },
    tokens::{
        token::{LiteralType, Token},
        token_type::TokenType,
    },
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

    #[inline]
    pub fn parse(&mut self) -> Vec<ParseResult<Stmt>> {
        let mut res = Vec::<ParseResult<Stmt>>::new();

        while !self.is_at_end() {
            res.push(self.declaration());
        }

        res
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        use crate::tokens::token_type::TokenType::Var;
        let res = if check!(self.peek(), Var) {
            self.advance();
            self.var_declaration()
        } else {
            self.statement()
        };

        res.or_else(|err| {
            self.synchronize();
            Err(err)
        })
    }
    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        use crate::tokens::token_type::TokenType::{Equal, Identifier, Semicolon};
        let name = self
            .consume(Identifier, "Expected variable name after var")?
            .clone();
        let mut init: Expr = LiteralType::Nil.into();
        if check!(self.peek(), Equal) {
            self.advance();
            init = self.expression()?;
        };
        self.consume(Semicolon, "Expected ';' after variable declaration")?;
        Ok(Stmt::Var(Var::new(name, init)))
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        use crate::tokens::token_type::TokenType::{If, LeftBrace, Print, While};
        if check!(self.peek(), If) {
            self.advance();
            self.if_statement()
        } else if check!(self.peek(), Print) {
            self.advance();
            self.print_statement()
        } else if check!(self.peek(), While) {
            self.advance();
            self.while_statement()
        } else if check!(self.peek(), LeftBrace) {
            self.advance();
            Ok(Block::new(self.block()?).into())
        } else {
            self.expression_statement()
        }
    }

    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        use crate::tokens::token_type::TokenType::RightBrace;
        let mut statements = Vec::new();
        while !check!(self.peek(), RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(RightBrace, "Expected '}' after initial block")?;
        Ok(statements)
    }
    fn if_statement(&mut self) -> ParseResult<Stmt> {
        use crate::tokens::token_type::TokenType::{Else, LeftParen, RightParen};

        self.consume(LeftParen, "Expected '(' after 'if'.")?;
        let cond = self.expression()?;
        self.consume(RightParen, "Expected ')' after 'if condition'.")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;
        if check!(self.peek(), Else) {
            self.advance();
            else_branch = Some(Box::new(self.statement()?));
        };

        return Ok(If::new(cond, Box::new(then_branch), else_branch).into());
    }

    #[inline]
    fn while_statement(&mut self) -> ParseResult<Stmt> {
        use crate::tokens::token_type::TokenType::{LeftParen, RightParen};

        self.consume(LeftParen, "Expected '(' after 'while'.")?;
        let cond = self.expression()?;
        self.consume(RightParen, "Expected ')' after 'while condition'.")?;
        let body = self.statement()?;
        Ok(While::new(cond, Box::new(body)).into())
    }
    #[inline]
    fn print_statement(&mut self) -> ParseResult<Stmt> {
        use crate::tokens::token_type::TokenType::Semicolon;

        let value = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value")?;
        Ok(Print::new(value).into())
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        use crate::tokens::token_type::TokenType::Semicolon;

        let expr = self.expression()?;
        self.consume(Semicolon, "Expected ';' after expression")?;
        Ok(Expression::new(expr).into())
    }
    #[inline]
    fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn and(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::And;

        let mut expr = self.equality()?;

        while check!(self.peek(), And) {
            let operator = self.advance().clone();
            let right = self.equality()?;
            expr = Logical::new(Box::new(expr), operator, Box::new(right)).into()
        }
        Ok(expr)
    }
    fn or(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::Or;

        let mut expr = self.and()?;

        while check!(self.peek(), Or) {
            let operator = self.advance().clone();
            let right = self.and()?;
            expr = Logical::new(Box::new(expr), operator, Box::new(right)).into();
        }
        Ok(expr)
    }
    fn assignment(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::Equal;
        let expr = self.or()?;

        if check!(self.peek(), Equal) {
            let equals = self.peek().unwrap().clone();
            self.advance();

            let value = self.assignment()?;
            return match &expr {
                Expr::Variable(var) => {
                    let name = &var.name;
                    Ok(Assign::new(name.clone(), Box::new(value)).into())
                }
                _ => Err(ParseError::new(
                    "Invalid l value for assignment",
                    (&equals).line,
                )),
            };
        }
        Ok(expr)
    }
    fn equality(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::{BangEqual, EqualEqual};
        let mut expr = self.comparision()?;
        while check!(self.peek(), EqualEqual | BangEqual) {
            let operator = (*self.advance()).clone();
            let right = self.comparision()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)))
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::{Greater, GreaterEqual, Less, LessEqual};

        let mut expr = self.term()?;

        while check!(self.peek(), Greater | GreaterEqual | Less | LessEqual) {
            let operator = (*self.advance()).clone();
            let right = self.term()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        Ok(expr)
    }
    fn term(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::{Minus, Plus};
        let mut expr = self.factor()?;

        while check!(self.peek(), Minus | Plus) {
            let operator = (*self.advance()).clone();
            let right = self.factor()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::{Slash, Star};
        let mut expr = self.unary()?;

        while check!(self.peek(), Slash | Star) {
            let operator = (*self.advance()).clone();
            let right = self.unary()?;
            expr = Expr::Binary(Binary::new(Box::new(expr), operator, Box::new(right)));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::{Bang, Minus};

        if check!(self.peek(), Bang | Minus) {
            let operator = (*self.advance()).clone();
            let right = self.unary()?;
            Ok(Expr::Unary(Unary::new(operator, Box::new(right))))
        } else {
            self.primary()
        }
    }
    fn primary(&mut self) -> ParseResult<Expr> {
        use crate::tokens::token_type::TokenType::{
            False, Identifier, LeftParen, Nil, Number, RightParen, Semicolon, String, True,
        };
        Ok(match self.advance().ty {
            False => LiteralType::False.into(),
            True => LiteralType::True.into(),
            Nil => LiteralType::Nil.into(),
            Number | String => self.previous().literal.clone().into(),
            Identifier => Expr::Variable(Variable::new(self.previous().clone())),
            LeftParen => {
                let expr = self.expression()?;
                self.consume(RightParen, "Expected ')' after expression")?;
                Grouping::new(Box::new(expr)).into()
            }
            Semicolon => {
                // found a semicolon
                // go back so we can consume it
                // this is likely an empty statement
                // this allows us to not crash when we get something like
                // print 1;;;
                self.current -= 1;
                LiteralType::InternalNoValue.into()
            }
            _ => {
                self.current -= 1; // did not match anything, backtrack
                let val = self.peek().unwrap();
                self.error::<Expr>(&val.clone(), "Expected expression")?
            }
        })
    }
    #[allow(dead_code)]
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
    fn error<T>(&mut self, t: &Token, err: &str) -> ParseResult<T> {
        Err(match t.ty {
            TokenType::EOF => ParseError::new(&format!("at the end: {err}"), t.line),
            _ => ParseError::new(&format!("at '{}': {}", t.lexeme, err), t.line),
        })
    }
    #[inline]
    fn consume(&mut self, x: TokenType, err: &str) -> ParseResult<&Token> {
        if self.peek().unwrap().ty == x {
            Ok(self.advance())
        } else {
            let token = &self.peek().unwrap().clone();
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
