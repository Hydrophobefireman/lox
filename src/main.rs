mod ast_printer;
mod parser;
mod program;
mod scanner;
mod tokens;
mod interpreter;
mod errors;
use std::{env, io};
mod expr;
use program::Program;

fn main() -> io::Result<()> {
    let mut p = Program::new();
    match env::args().len() {
        1 => p.repl(),
        2 => p.run_script(env::args().nth(1).unwrap().to_string()),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "usage: lox [file]",
        )),
    }
}
