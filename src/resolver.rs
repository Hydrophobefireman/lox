use std::collections::HashMap;

use crate::{
    errors::{ResolverError, ResolverResult},
    interpreter::Interpreter,
    syntax::{
        expr::{self, Expr},
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
        stmt.accept(self)
    }

    fn resolve_expr(self, e: Expr) -> ResolverResult<Expr> {
        e.accept(self)
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

impl expr::Visitor<ResolverResult<Expr>> for Resolver {
    fn Variable(mut self, e: expr::Variable) -> ResolverResult<Expr> {
        if let Some(el) = self.scopes.last_mut() {
            if let Some(val) = el.get(&e.name.lexeme) {
                if !val {
                    return Err(ResolverError::new(
                        "Can't read local variable in its own initializer".into(),
                        e.name.line,
                        self.interpreter,
                    ));
                }
            }
        }

        let t = &e.name.clone();
        let e = self.resolve_local(e.into(), t);
        if let Expr::Variable(e) = e {
            Ok((e.into(), self))
        } else {
            panic!("??")
        }
    }
    fn Assign(self, mut e: expr::Assign) -> ResolverResult<Expr> {
        let mut this = self;
        (*e.value, this) = this.resolve_expr(*e.value)?;
        let t = &e.name.clone();
        let e = this.resolve_local(e.into(), t);
        if let Expr::Assign(e) = e {
            Ok((e.into(), this))
        } else {
            panic!("??")
        }
    }
    fn Binary(self, mut e: expr::Binary) -> ResolverResult<Expr> {
        let mut this = self;
        (*e.left, this) = this.resolve_expr(*e.left)?;
        (*e.right, this) = this.resolve_expr(*e.right)?;

        Ok((e.into(), this))
    }
    fn Call(self, e: expr::Call) -> ResolverResult<Expr> {
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
    fn Grouping(self, e: expr::Grouping) -> ResolverResult<Expr> {
        let (res, this) = self.resolve_expr(*e.expression)?;
        if let Expr::Grouping(res) = res {
            return Ok((res.into(), this));
        } else {
            panic!("??")
        }
    }
    fn Literal(self, l: expr::Literal) -> ResolverResult<Expr> {
        Ok((l.into(), self))
    }
    fn Logical(self, mut e: expr::Logical) -> ResolverResult<Expr> {
        let mut this = self;
        (*e.left, this) = this.resolve_expr(*e.left)?;
        (*e.right, this) = this.resolve_expr(*e.right)?;
        Ok((e.into(), this))
    }
    fn Unary(self, mut e: expr::Unary) -> ResolverResult<Expr> {
        let this;
        (*e.right, this) = self.resolve_expr(*e.right)?;
        Ok((e.into(), this))
    }
}

impl stmt::Visitor<ResolverResult<Stmt>> for Resolver {
    fn Block(self, mut e: stmt::Block) -> ResolverResult<Stmt> {
        let mut this = self;
        this.begin_scope();
        let stmts;
        (stmts, this) = this.resolve_statements(e.statements)?;
        this.end_scope();
        e.statements = stmts;
        Ok((e.into(), this))
    }
    fn Expression(self, mut e: stmt::Expression) -> ResolverResult<Stmt> {
        let (expr, this) = self.resolve_expr(e.expression)?;
        e.expression = expr;
        Ok((e.into(), this))
    }
    fn If(self, mut e: stmt::If) -> ResolverResult<Stmt> {
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

    fn Print(self, mut e: stmt::Print) -> ResolverResult<Stmt> {
        let (expr, this) = self.resolve_expr(e.expression)?;
        e.expression = expr;
        Ok((e.into(), this))
    }
    fn Return(self, mut e: stmt::Return) -> ResolverResult<Stmt> {
        if let Some(val) = e.value {
            let (expr, this) = self.resolve_expr(val)?;
            e.value = Some(expr);
            Ok((e.into(), this))
        } else {
            Ok((e.into(), self))
        }
    }
    fn While(self, mut e: stmt::While) -> ResolverResult<Stmt> {
        let mut this = self;
        let cond;
        let body;
        (cond, this) = this.resolve_expr(e.cond)?;
        (body, this) = this.resolve_stmt(*e.body)?;
        e.cond = cond;
        e.body = Box::new(body);
        Ok((e.into(), this))
    }

    fn Function(mut self, e: stmt::Function) -> ResolverResult<Stmt> {
        self.declare(&e.name);
        self.define(&e.name);
        let (e, this) = self.resolve_function(e)?;
        Ok((e.into(), this))
    }
    fn Var(mut self, mut e: stmt::Var) -> ResolverResult<Stmt> {
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
}
