use std::fs;
use std::io::{self, BufRead, Write};
use std::process::exit;

use crate::scanner::Scanner;

pub struct Program {
    had_error: bool,
}
impl Program {
    pub fn new() -> Program {
        Program { had_error: false }
    }
}
impl Program {
    fn run(&self, line: &str) {
        let mut scanner = Scanner::new(line, self);
        let tokens = scanner.scan_tokens();

        for token in tokens {
            dbg!(token);
        }
    }

    pub fn error(&self, line: usize, message: &str) {
        self.report(line, "", message);
    }
    fn report(&self, line: usize, wh: &str, message: &str) {
        println!("[line {line}] Error{wh}: {message}");
    }

    pub fn repl(&mut self) -> io::Result<()> {
        let input = io::stdin();
        let mut reader = input.lock();
        Ok(loop {
            print!("> ");
            io::stdout().flush()?;
            let mut line = String::new();
            reader.read_line(&mut line)?;
            if line.is_empty() {
                break;
            }
            self.run(line.trim());
            io::stdout().flush()?;
            self.had_error = false
        })
    }

    pub fn run_script(&self, file: String) -> io::Result<()> {
        let content = fs::read_to_string(file)?;

        self.run(&content);
        if self.had_error {
            exit(65);
        }
        Ok(())
    }
}
