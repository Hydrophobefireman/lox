use std::collections::HashMap;

use crate::{
    errors::{ResolverError, ResolverResult},
    interpreter::Interpreter,
    syntax::{
        expr::Expr,
        stmt::{self, Stmt},
    },
    tokens::token::{LoxType, Token},
};

#[derive(Debug)]
pub struct Resolver {
    pub interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>,
}

impl Resolver {
    pub fn new(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Default::default(),
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Default::default())
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }
    pub fn resolve_statements(self, stmts: Vec<Stmt>) -> ResolverResult<Vec<Stmt>> {
        let mut res = Vec::new();
        let mut this = self;
        for stmt in stmts {
            let st;
            (st, this) = this.resolve_stmt(stmt)?;
            res.push(st);
        }

        Ok((res, this))
    }
    fn resolve_stmt(self, stmt: Stmt) -> ResolverResult<Stmt> {
        match stmt {
            Stmt::Block(e) => self.handle_block_stmt(e),
            Stmt::Expression(e) => self.handle_expr_stmt(e),
            Stmt::Function(e) => self.resolve_function_stmt(e),
            Stmt::If(e) => self.handle_if_stmt(e),
            Stmt::Print(e) => self.handle_print_stmt(e),
            Stmt::Return(ret) => self.handle_return_stmt(ret),
            Stmt::Var(v) => self.handle_var_stmt(v),
            Stmt::While(wh) => self.handle_while_stmt(wh),
        }
    }

    fn handle_while_stmt(self, mut e: stmt::While) -> ResolverResult<Stmt> {
        let mut this = self;
        let cond;
        let body;
        (cond, this) = this.resolve_expr(e.cond)?;
        (body, this) = this.resolve_stmt(*e.body)?;
        e.cond = cond;
        e.body = Box::new(body);
        Ok((e.into(), this))
    }

    fn handle_var_stmt(mut self, mut e: stmt::Var) -> ResolverResult<Stmt> {
        self.declare(&e.name);
        let (init, mut this) = match &e.initializer {
            Expr::Literal(lit) => {
                if !matches!(lit.value, LoxType::Nil) {
                    self.resolve_expr(e.initializer)?
                } else {
                    (e.initializer, self)
                }
            }
            _ => self.resolve_expr(e.initializer)?,
        };
        this.define(&e.name);
        e.initializer = init;
        Ok((e.into(), this))
    }

    fn handle_return_stmt(self, mut e: stmt::Return) -> ResolverResult<Stmt> {
        if let Some(val) = e.value {
            let (expr, this) = self.resolve_expr(val)?;
            e.value = Some(expr);
            Ok((e.into(), this))
        } else {
            Ok((e.into(), self))
        }
    }
    fn handle_print_stmt(self, mut e: stmt::Print) -> ResolverResult<Stmt> {
        let (expr, this) = self.resolve_expr(e.expression)?;
        e.expression = expr;
        Ok((e.into(), this))
    }
    fn handle_if_stmt(self, mut e: stmt::If) -> ResolverResult<Stmt> {
        let mut this = self;
        let cond;
        let then;
        let else_;
        (cond, this) = this.resolve_expr(e.cond)?;
        (then, this) = this.resolve_stmt(*e.then_branch)?;
        e.cond = cond;
        e.then_branch = Box::new(then);
        if let Some(else_branch) = e.else_branch {
            (else_, this) = this.resolve_stmt(*else_branch)?;
            e.else_branch = Some(Box::new(else_));
        }
        Ok((e.into(), this))
    }

    fn resolve_function_stmt(mut self, e: stmt::Function) -> ResolverResult<Stmt> {
        self.declare(&e.name);
        self.define(&e.name);
        let (e, this) = self.resolve_function(e)?;
        Ok((e.into(), this))
    }

    fn handle_expr_stmt(self, mut e: stmt::Expression) -> ResolverResult<Stmt> {
        let (expr, this) = self.resolve_expr(e.expression)?;
        e.expression = expr;
        Ok((e.into(), this))
    }

    fn handle_block_stmt(self, mut e: stmt::Block) -> ResolverResult<Stmt> {
        let mut this = self;
        this.begin_scope();
        let stmts;
        (stmts, this) = this.resolve_statements(e.statements)?;
        this.end_scope();
        e.statements = stmts;
        Ok((e.into(), this))
    }

    fn resolve_expr(self, e: Expr) -> ResolverResult<Expr> {
        match e {
            Expr::Assign(mut e) => {
                let mut this = self;
                (*e.value, this) = this.resolve_expr(*e.value)?;
                let t = &e.name.clone();
                let e = this.resolve_local(e.into(), t);
                Ok((e, this))
            }
            Expr::Binary(mut e) => {
                let mut this = self;
                (*e.left, this) = this.resolve_expr(*e.left)?;
                (*e.right, this) = this.resolve_expr(*e.right)?;
                Ok((e.into(), this))
            }
            Expr::Call(e) => {
                let (e, mut this) = self.resolve_expr(*e.callee)?;
                match e {
                    Expr::Call(mut e) => {
                        let mut args = Vec::new();
                        for arg in e.args {
                            let arg_;
                            (arg_, this) = this.resolve_expr(arg)?;
                            args.push(arg_);
                        }
                        e.args = args;
                        Ok((e.into(), this))
                    }
                    _ => panic!("??"),
                }
            }
            Expr::Grouping(e) => {
                let (res, this) = self.resolve_expr(*e.expression)?;
                if let Expr::Grouping(res) = res {
                    return Ok((res.into(), this));
                } else {
                    panic!("??")
                }
            }
            Expr::Literal(e) => Ok((e.into(), self)),
            Expr::Logical(mut e) => {
                let mut this = self;
                (*e.left, this) = this.resolve_expr(*e.left)?;

                (*e.right, this) = this.resolve_expr(*e.right)?;
                return Ok((e.into(), this));
            }
            Expr::Unary(mut e) => {
                let this;
                (*e.right, this) = self.resolve_expr(*e.right)?;
                Ok((e.into(), this))
            }
            Expr::Variable(e) => {
                let mut this = self;
                if let Some(el) = this.scopes.last_mut() {
                    if let Some(val) = el.get(&e.name.lexeme) {
                        if !val {
                            return Err(ResolverError::new(
                                "Can't read local variable in its own initializer".into(),
                                e.name.line,
                                this.interpreter,
                            ));
                        }
                    }
                }

                let t = &e.name.clone();
                let e = this.resolve_local(e.into(), t);
                Ok((e, this))
            }
        }
    }

    fn declare(&mut self, name: &Token) {
        self.scopes
            .last_mut()
            .map(|top| top.insert(name.lexeme.clone(), false));
    }

    fn define(&mut self, name: &Token) {
        self.scopes
            .last_mut()
            .map(|top| top.insert(name.lexeme.clone(), true));
    }
    fn resolve_local(&self, mut e: Expr, name: &Token) -> Expr {
        if let Some((_, i)) = self
            .scopes
            .iter()
            .rev()
            .zip(0..)
            .find(|(scope, _)| scope.contains_key(&name.lexeme))
        {
            e.depth(i);
        };
        e
        // .map(|(_, i)| self.interpreter.resolve(e, i));
    }

    fn resolve_function(mut self, mut fun: stmt::Function) -> ResolverResult<Stmt> {
        self.begin_scope();
        (&fun.params).into_iter().for_each(|param| {
            self.declare(param);
            self.define(param);
        });
        let (stmts, mut this) = self.resolve_statements(fun.body)?;
        this.end_scope();
        fun.body = stmts;
        Ok((fun.into(), this))
    }
}
