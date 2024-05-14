mod ast_printer;
mod errors;
mod globals;
mod interpreter;
mod lox_class;
mod lox_function;
mod parser;
mod program;
mod resolver;
mod scanner;
mod tokens;
use std::{env, io};
mod environment;
mod syntax;
use program::Program;

fn main() -> io::Result<()> {
    let p = Program::new();
    match env::args().len() {
        1 => p.repl(),
        2 => p.run_script(env::args().nth(1).unwrap().to_string()),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "usage: lox [file]",
        )),
    }
}
