mod program;
mod scanner;
mod token;
mod token_type;
use std::{env, io};

use program::Program;
fn main() -> io::Result<()> {
    let mut p = Program::new();
    match env::args().len() {
        1 => p.repl(),
        2 => p.run_script(env::args().nth(1).unwrap().to_string()),
        _ => panic!("usage: lox [file]"),
    }
}
