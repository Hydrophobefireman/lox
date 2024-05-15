use crate::{
    errors::{ParseError, ParseResult},
    syntax::{
        expr::{
            self, Assign, Binary, Call, Expr, Get, Grouping, Literal, Logical, Set, Super, Unary,
            Variable,
        },
        stmt::{Block, Class, Expression, Function, If, Print, Return, Stmt, Var, While},
    },
    tokens::{
        token::{LoxCallableType, LoxType, Token},
        token_type::TokenType,
    },
};

pub struct Parser {
    tokens: Vec<Token>,
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

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { current: 0, tokens }
    }

    pub fn parse(&mut self) -> Vec<ParseResult<Stmt>> {
        let mut res = Vec::<ParseResult<Stmt>>::new();

        while !self.is_at_end() {
            res.push(self.declaration());
        }

        res
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        use TokenType::{Class, Fun, Var};

        let res = if check!(self.peek(), Class) {
            self.advance();
            self.class_declaration()
        } else if check!(self.peek(), Fun) {
            self.advance();
            self.function(LoxCallableType::Function)
        } else {
            if check!(self.peek(), Var) {
                self.advance();
                self.var_declaration()
            } else {
                self.statement()
            }
        };

        res.or_else(|err| {
            self.synchronize();
            Err(err)
        })
    }
    fn class_declaration(&mut self) -> ParseResult<Stmt> {
        use TokenType::{Identifier, LeftBrace, Less, RightBrace};

        let name = self.consume(Identifier, "Expected class name")?.clone();
        let superclass = if check!(self.peek(), Less) {
            self.advance();
            let sc = self
                .consume(Identifier, "Expected superclass class name")?
                .clone();
            Some(expr::Variable::new(sc, None))
        } else {
            None
        };

        self.consume(LeftBrace, "Expected '{' before class body")?;
        let mut methods = Vec::new();
        while !self.is_at_end() && !check!(self.peek(), RightBrace) {
            let fun = self.function(LoxCallableType::Class)?;
            match fun {
                Stmt::Function(fun) => {
                    methods.push(fun);
                }
                _ => panic!("Unexpected statement in class body"),
            }
        }
        self.advance();
        return Ok(Class::new(name, superclass, methods).into());
    }
    fn function(&mut self, kind: LoxCallableType) -> ParseResult<Stmt> {
        use TokenType::{Comma, Identifier, LeftBrace, LeftParen, RightParen};

        let name = self
            .consume(Identifier, &format!("Expected {:?} name.", kind))?
            .clone();
        self.consume(LeftParen, &format!("Expected '(' after {:?} name.", kind))?;
        let mut params = Vec::new();
        if !check!(self.peek(), RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(ParseError::new(
                        "Can't have more than 255 params",
                        self.peek().unwrap().line,
                    ));
                }

                params.push(self.consume(Identifier, "Expected paramter name")?.clone());
                if check!(self.peek(), Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        };
        self.consume(RightParen, "Expected ')' after parameters")?;
        self.consume(LeftBrace, &format!("Expected '{{' before {:?} body", kind))?;
        let body = self.block()?;
        return Ok(Function::new(name, params, body).into());
    }
    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        use TokenType::{Equal, Identifier, Semicolon};
        let name = self
            .consume(Identifier, "Expected variable name after var")?
            .clone();
        let mut init: Expr = LoxType::Nil.into();
        if check!(self.peek(), Equal) {
            self.advance();
            init = self.expression()?;
        };
        self.consume(Semicolon, "Expected ';' after variable declaration")?;
        Ok(Var::new(name, init).into())
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::{For, If, LeftBrace, Print, Return, While};

        if check!(self.peek(), For) {
            self.advance();
            self.for_statement()
        } else if check!(self.peek(), If) {
            self.advance();
            self.if_statement()
        } else if check!(self.peek(), Print) {
            self.advance();
            self.print_statement()
        } else if check!(self.peek(), Return) {
            self.advance();
            self.return_statement()
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
        use TokenType::RightBrace;
        let mut statements = Vec::new();
        while !check!(self.peek(), RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(RightBrace, "Expected '}' after initial block")?;
        Ok(statements)
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::{LeftParen, RightParen, Semicolon, Var};

        self.consume(LeftParen, "Expect '(' after 'for'.")?;
        let mut initializer = None;
        if check!(self.peek(), Semicolon) {
            self.advance();
        } else if check!(self.peek(), Var) {
            self.advance();
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_statement()?);
        }
        let mut condition = None;
        if !check!(self.peek(), Semicolon) {
            condition = Some(self.expression()?);
        }

        self.consume(Semicolon, "Expected ';' after loop condition")?;

        let mut increment = None;
        if !check!(self.peek(), RightParen) {
            increment = Some(self.expression()?);
        }
        self.consume(RightParen, "Expected ')' after for clauses")?;
        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Block::new(vec![body, Expression::new(increment).into()]).into();
        }
        let condition = match condition {
            Some(v) => v,
            None => Literal::new(true.into(), None).into(),
        };

        body = While::new(condition, Box::new(body)).into();

        if let Some(initializer) = initializer {
            body = Block::new(vec![initializer, body]).into();
        }
        Ok(body)
    }
    fn if_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::{Else, LeftParen, RightParen};

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
    fn return_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::Semicolon;
        let token = self.previous().clone();
        let mut value = None;
        if !check!(self.peek(), Semicolon) {
            value = Some(self.expression()?);
        };
        self.consume(Semicolon, "Expected ';' after return value.")?;
        return Ok(Return::new(token, value).into());
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::{LeftParen, RightParen};

        self.consume(LeftParen, "Expected '(' after 'while'.")?;
        let cond = self.expression()?;
        self.consume(RightParen, "Expected ')' after 'while condition'.")?;
        let body = self.statement()?;
        Ok(While::new(cond, Box::new(body)).into())
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::Semicolon;

        let value = self.expression()?;
        self.consume(Semicolon, "Expected ';' after value")?;
        Ok(Print::new(value).into())
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        use TokenType::Semicolon;

        let expr = self.expression()?;
        self.consume(Semicolon, "Expected ';' after expression")?;
        Ok(Expression::new(expr).into())
    }

    fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    fn and(&mut self) -> ParseResult<Expr> {
        use TokenType::And;

        let mut expr = self.equality()?;

        while check!(self.peek(), And) {
            let operator = self.advance().clone();
            let right = self.equality()?;
            expr = Logical::new(Box::new(expr), operator, Box::new(right), None).into()
        }
        Ok(expr)
    }
    fn or(&mut self) -> ParseResult<Expr> {
        use TokenType::Or;

        let mut expr = self.and()?;

        while check!(self.peek(), Or) {
            let operator = self.advance().clone();
            let right = self.and()?;
            expr = Logical::new(Box::new(expr), operator, Box::new(right), None).into();
        }
        Ok(expr)
    }
    fn assignment(&mut self) -> ParseResult<Expr> {
        use TokenType::Equal;
        let expr = self.or()?;

        if check!(self.peek(), Equal) {
            let equals = self.peek().unwrap().clone();
            self.advance();

            let value = self.assignment()?;
            return match &expr {
                Expr::Variable(var) => {
                    Ok(Assign::new(var.name.clone(), Box::new(value), None).into())
                }
                Expr::Get(get) => Ok(Set::new(
                    Box::clone(&get.object),
                    get.name.clone(),
                    Box::new(value),
                    None,
                )
                .into()),
                _ => Err(ParseError::new(
                    "Invalid l value for assignment",
                    (&equals).line,
                )),
            };
        }
        Ok(expr)
    }
    fn equality(&mut self) -> ParseResult<Expr> {
        use TokenType::{BangEqual, EqualEqual};
        let mut expr = self.comparision()?;
        while check!(self.peek(), EqualEqual | BangEqual) {
            let operator = (*self.advance()).clone();
            let right = self.comparision()?;
            expr = Binary::new(Box::new(expr), operator, Box::new(right), None).into()
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> ParseResult<Expr> {
        use TokenType::{Greater, GreaterEqual, Less, LessEqual};

        let mut expr = self.term()?;

        while check!(self.peek(), Greater | GreaterEqual | Less | LessEqual) {
            let operator = (*self.advance()).clone();
            let right = self.term()?;
            expr = Binary::new(Box::new(expr), operator, Box::new(right), None).into();
        }
        Ok(expr)
    }
    fn term(&mut self) -> ParseResult<Expr> {
        use TokenType::{Minus, Plus};
        let mut expr = self.factor()?;

        while check!(self.peek(), Minus | Plus) {
            let operator = (*self.advance()).clone();
            let right = self.factor()?;
            expr = Binary::new(Box::new(expr), operator, Box::new(right), None).into();
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expr> {
        use TokenType::{Slash, Star};
        let mut expr = self.unary()?;

        while check!(self.peek(), Slash | Star) {
            let operator = (*self.advance()).clone();
            let right = self.unary()?;
            expr = Binary::new(Box::new(expr), operator, Box::new(right), None).into();
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expr> {
        use TokenType::{Bang, Minus};

        if check!(self.peek(), Bang | Minus) {
            let operator = (*self.advance()).clone();
            let right = self.unary()?;
            Ok(Unary::new(operator, Box::new(right), None).into())
        } else {
            self.call()
        }
    }
    fn call(&mut self) -> ParseResult<Expr> {
        use TokenType::{Dot, Identifier, LeftParen};

        let mut expr = self.primary()?;
        loop {
            if check!(self.peek(), LeftParen) {
                self.advance();
                expr = self.handle_call(expr)?;
            } else if check!(self.peek(), Dot) {
                self.advance();
                let name: &Token = self.consume(Identifier, "Expected property name after '.'")?;
                expr = Get::new(Box::new(expr), name.clone(), None).into()
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn handle_call(&mut self, callee: Expr) -> ParseResult<Expr> {
        use TokenType::{Comma, RightParen};

        let mut args = Vec::new();

        if !check!(self.peek(), RightParen) {
            loop {
                if args.len() >= 255 {
                    return Err(ParseError::new(
                        "Can't have more than 255 arguments",
                        self.peek().unwrap().line,
                    ));
                }
                args.push(self.expression()?);
                if check!(self.peek(), Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        let paren = self.consume(RightParen, "Expect ')' after arguments.")?;
        Ok(Call::new(Box::new(callee), paren.clone(), args, None).into())
    }
    fn primary(&mut self) -> ParseResult<Expr> {
        use TokenType::{
            Dot, False, Identifier, LeftParen, Nil, Number, RightParen, Semicolon, String, This,
            True,
        };
        Ok(match self.advance().ty {
            False => LoxType::False.into(),
            True => LoxType::True.into(),
            Nil => LoxType::Nil.into(),
            Number | String => self.previous().literal.clone().into(),
            Identifier => Variable::new(self.previous().clone(), None).into(),
            This => expr::This::new(self.previous().clone(), None).into(),
            LeftParen => {
                let expr = self.expression()?;
                self.consume(RightParen, "Expected ')' after expression")?;
                Grouping::new(Box::new(expr), None).into()
            }
            TokenType::Super => {
                let kw = self.previous().clone();
                self.consume(Dot, "Expect a . after 'super'")?;
                let method = self
                    .consume(Identifier, "Expect super method name")?
                    .clone();
                Super::new(kw, method, None).into()
            }
            Semicolon => {
                // found a semicolon
                // go back so we can consume it
                // this is likely an empty statement
                // this allows us to not crash when we get something like
                // print 1;;;
                self.current -= 1;
                LoxType::InternalNoValue.into()
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

    fn error<T>(&mut self, t: &Token, err: &str) -> ParseResult<T> {
        Err(match t.ty {
            TokenType::EOF => ParseError::new(format!("at the end: {err}"), t.line),
            _ => ParseError::new(format!("at '{}': {}", t.lexeme, err), t.line),
        })
    }

    #[must_use = "consume causes parse errors when failed which needs to be reconciled"]
    fn consume(&mut self, x: TokenType, err: &str) -> ParseResult<&Token> {
        if self.peek().unwrap().ty == x {
            Ok(self.advance())
        } else {
            let token = &self.peek().unwrap().clone();
            self.error(token, err)
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn is_at_end(&self) -> bool {
        check!(self.peek(), TokenType::EOF)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        };
        self.previous()
    }
}
