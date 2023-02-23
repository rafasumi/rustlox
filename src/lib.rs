mod ast;
mod error;
mod parser;
mod scanner;
mod token;
mod interpreter;

use error::Error;
use parser::Parser;
use scanner::Scanner;
use std::{
    fs,
    io::{self, Write},
    process,
};

use crate::interpreter::Interpreter;

fn run(source: &str) -> Result<(), Error> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&expr)?;

    Ok(())
}

pub fn run_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(file_path)?;
    if let Err(error) = run(&source) {
        match error {
            Error::Runtime {..} => process::exit(70),
            _ => process::exit(65)
        }
    }

    Ok(())
}

pub fn run_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let mut line = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        print!("> ");
        stdout.flush()?;

        let n = stdin.read_line(&mut line)?;
        if n == 0 {
            break;
        }

        run(&line).ok();
        line.clear();
    }

    Ok(())
}
