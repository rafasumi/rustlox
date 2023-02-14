mod ast;
mod error;
mod parser;
mod scanner;
mod token;

use ast::{AstPrinter, Visitor};
use parser::Parser;
use scanner::Scanner;
use std::{
    error::Error,
    fs,
    io::{self, Write},
    process,
};

fn run(source: &str) -> Result<(), ()> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    let mut printer = AstPrinter;
    println!("{}", printer.visit_expr(&expr));

    Ok(())
}

pub fn run_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(file_path)?;
    if let Err(_) = run(&source) {
        process::exit(65);
    }

    Ok(())
}

pub fn run_prompt() -> Result<(), Box<dyn Error>> {
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
