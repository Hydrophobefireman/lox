mod ast_printer;
mod errors;
mod interpreter;
mod parser;
mod program;
mod scanner;
mod tokens;
mod globals;
mod lox_function;
use std::{env, io};
mod environment;
mod syntax;
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
