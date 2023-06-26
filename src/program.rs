use std::fs;
use std::io::{self, BufRead, Write};
use std::process::exit;

use crate::interpreter::Interpreter;
use crate::parser::Parser;
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
    fn run(&mut self, line: &str) -> LoxType {
        if line.is_empty() {
            return LoxType::InternalNoValue;
        }
        let scanner = Scanner::new(line);
        let tokens = scanner.scan_tokens();

        match tokens {
            Ok(tokens) => {
                let mut parser = Parser::new(&tokens);
                let stmts = parser.parse();

                if stmts.iter().any(|f| f.is_err()) {
                    self.had_error = true;
                    stmts
                        .iter()
                        .filter_map(|f| f.as_ref().err())
                        .for_each(|err| self.error(err.line, &err.message));
                    return Default::default();
                }

                let stmts: Vec<_> = stmts.into_iter().map(Result::unwrap).collect();

                match self.interpreter.interpret(&stmts) {
                    Err(r) => {
                        self.runtime_error(0, &r.message);
                        LoxType::InternalNoValue
                    }
                    Ok(v) => v,
                }
            }
            Err(err) => {
                self.error(err.line, &err.message);
                LoxType::InternalNoValue
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

    pub fn repl(&mut self) -> io::Result<()> {
        let input = io::stdin();
        let mut reader = input.lock();
        loop {
            print!("> ");
            io::stdout().flush()?;
            let mut line = String::new();
            reader.read_line(&mut line)?;
            if line.is_empty() {
                break Ok(());
            }
            let res = self.run(line.trim());
            if !matches!(res, LoxType::InternalNoValue) {
                println!("{}", res.to_string());
            }
            io::stdout().flush()?;
            self.had_error = false;
            self.had_runtime_error = false;
        }
    }

    pub fn run_script(&mut self, file: String) -> io::Result<()> {
        let content = fs::read_to_string(file)?;

        self.run(&content);
        if self.had_error {
            exit(65);
        }
        if self.had_runtime_error {
            exit(70);
        }
        Ok(())
    }
}
