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

pub struct Resolver {
    interpreter: Interpreter,
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
    fn resolve_statements(&mut self, stmts: &Vec<Stmt>) -> ResolverResult<()> {
        stmts
            .iter()
            .map(|s| self.resolve_stmt(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
    fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolverResult<()> {
        stmt.accept(self)
    }

    fn resolve_expr(&mut self, e: &Expr) -> ResolverResult<()> {
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
    fn resolve_local(&mut self, e: &Expr, name: &Token) {
        self.scopes
            .iter()
            .rev()
            .zip(0..)
            .find(|(scope, _)| scope.contains_key(&name.lexeme))
            .map(|(_, i)| self.interpreter.resolve(e, i));
    }

    fn resolve_function(&mut self, fun: &stmt::Function) -> ResolverResult<()> {
        self.begin_scope();
        (&fun.params).into_iter().for_each(|param| {
            self.declare(param);
            self.define(param);
        });
        self.resolve_statements(&fun.body)?;
        self.end_scope();
        Ok(())
    }
}

impl expr::Visitor<ResolverResult<()>> for Resolver {
    fn Variable(&mut self, e: &expr::Variable) -> ResolverResult<()> {
        if let Some(el) = self.scopes.last_mut() {
            if let Some(val) = el.get(&e.name.lexeme) {
                if !val {
                    return Err(ResolverError::new(
                        "Can't read local variable in its own initializer",
                        e.name.line,
                    ));
                }
            }
        }
        self.resolve_local(&(e.clone().into()), &e.name);
        Ok(())
    }
    fn Assign(&mut self, e: &expr::Assign) -> ResolverResult<()> {
        self.resolve_expr(&e.value)?;
        self.resolve_local(&(e.clone().into()), &e.name);
        Ok(())
    }
    fn Binary(&mut self, e: &expr::Binary) -> ResolverResult<()> {
        self.resolve_expr(&e.left)?;
        self.resolve_expr(&e.right)?;

        Ok(())
    }
    fn Call(&mut self, e: &expr::Call) -> ResolverResult<()> {
        self.resolve_expr(&e.callee)?;
        e.args
            .iter()
            .map(|arg| self.resolve_expr(arg))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
    fn Grouping(&mut self, e: &expr::Grouping) -> ResolverResult<()> {
        self.resolve_expr(&e.expression)
    }
    fn Literal(&mut self, _: &expr::Literal) -> ResolverResult<()> {
        Ok(())
    }
    fn Logical(&mut self, e: &expr::Logical) -> ResolverResult<()> {
        self.resolve_expr(&e.left)?;
        self.resolve_expr(&e.right)?;
        Ok(())
    }
    fn Unary(&mut self, e: &expr::Unary) -> ResolverResult<()> {
        self.resolve_expr(&e.right)?;
        Ok(())
    }
}

impl stmt::Visitor<ResolverResult<()>> for Resolver {
    fn Block(&mut self, e: &stmt::Block) -> ResolverResult<()> {
        self.begin_scope();
        self.resolve_statements(&e.statements)?;
        self.end_scope();
        Ok(())
    }
    fn Expression(&mut self, e: &stmt::Expression) -> ResolverResult<()> {
        self.resolve_expr(&e.expression)
    }
    fn If(&mut self, e: &stmt::If) -> ResolverResult<()> {
        self.resolve_expr(&e.cond)?;
        self.resolve_stmt(&e.then_branch)?;

        if let Some(else_branch) = &e.else_branch {
            self.resolve_stmt(else_branch)?;
        }
        Ok(())
    }

    fn Print(&mut self, e: &stmt::Print) -> ResolverResult<()> {
        self.resolve_expr(&e.expression)
    }
    fn Return(&mut self, e: &stmt::Return) -> ResolverResult<()> {
        if let Some(val) = &e.value {
            self.resolve_expr(val)?;
        }
        Ok(())
    }
    fn While(&mut self, e: &stmt::While) -> ResolverResult<()> {
        self.resolve_expr(&e.cond)?;
        self.resolve_stmt(&e.body)?;
        Ok(())
    }

    fn Function(&mut self, e: &stmt::Function) -> ResolverResult<()> {
        self.declare(&e.name);
        self.define(&e.name);
        self.resolve_function(e)?;
        Ok(())
    }
    fn Var(&mut self, e: &stmt::Var) -> ResolverResult<()> {
        self.declare(&e.name);

        match &e.initializer {
            Expr::Literal(lit) => {
                if !matches!(lit.value, LoxType::Nil) {
                    self.resolve_expr(&e.initializer)?;
                }
            }
            _ => self.resolve_expr(&e.initializer)?,
        }
        self.define(&e.name);
        Ok(())
    }
}
