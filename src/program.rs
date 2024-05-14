use std::fs;
use std::io::{self, BufRead, Write};
use std::process::exit;

use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::scanner::Scanner;
use crate::tokens::token::LoxType;

pub struct Program {
    had_error: bool,
    had_runtime_error: bool,
    interpreter: Interpreter,
}
impl Program {
    pub fn new() -> Program {
        Program {
            had_error: false,
            had_runtime_error: false,
            interpreter: Interpreter::new(),
        }
    }
}
impl Program {
    fn run(mut self, line: String) -> (Self, LoxType) {
        if line.is_empty() {
            return (self, Default::default());
        }
        let scanner = Scanner::new(line);
        let tokens = scanner.scan_tokens();

        match tokens {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                let stmts = parser.parse();

                if stmts.iter().any(|f| f.is_err()) {
                    self.had_error = true;
                    stmts
                        .iter()
                        .filter_map(|f| f.as_ref().err())
                        .for_each(|err| self.error(err.line, &err.message));
                    return (self, Default::default());
                }

                let mut stmts = stmts.into_iter().map(Result::unwrap).collect::<Vec<_>>();
                let interpreter = self.interpreter;
                let resolver = Resolver::new(interpreter);
                match resolver.resolve_statements(stmts) {
                    Ok((i, b)) => {
                        stmts = i;
                        self.interpreter = b.interpreter;
                    }
                    Err(e) => {
                        self.interpreter = e.interpreter;
                        self.error(e.line, &e.message);
                        return (self, Default::default());
                    }
                }

                match self.interpreter.interpret(stmts) {
                    Err(r) => {
                        self.interpreter = r.interpreter;
                        self.runtime_error(0, &r.message);
                        (self, Default::default())
                    }
                    Ok(v) => {
                        self.interpreter = v.1;
                        (self, v.0)
                    }
                }
            }
            Err(err) => {
                self.error(err.line, &err.message);
                (self, Default::default())
            }
        }
    }

    pub fn error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
        self.had_error = true;
    }
    pub fn report(&self, line: usize, wh: &str, message: &str) {
        println!("[line {line}] Error{wh}: {message}");
    }

    fn runtime_error(&mut self, line: usize, message: &str) {
        self.report(line, "", message);
        self.had_runtime_error = true;
    }

    pub fn repl(self) -> io::Result<()> {
        let input = io::stdin();
        let mut reader = input.lock();
        let mut this = self;
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut line = String::new();
            reader.read_line(&mut line)?;
            if line.is_empty() {
                break Ok(());
            }
            let res;
            (this, res) = this.run(line);
            if !matches!(res, LoxType::InternalNoValue) {
                println!("{}", res.to_string());
            }
            io::stdout().flush()?;
            this.had_error = false;
            this.had_runtime_error = false;
        }
    }

    pub fn run_script(self, file: String) -> io::Result<()> {
        let content = fs::read_to_string(file)?;
        let mut this = self;
        this = this.run(content).0;
        if this.had_error {
            exit(65);
        }
        if this.had_runtime_error {
            exit(70);
        }
        Ok(())
    }
}
