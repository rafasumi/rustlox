mod ast;
mod callable;
mod environment;
mod error;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod token;

use error::Error;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;
use std::{
    fs,
    io::{self, Write},
    process,
};

use crate::interpreter::Interpreter;

pub struct RustLox {
    interpreter: Interpreter,
}

impl RustLox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    fn run(&mut self, source: &str) -> Result<(), Error> {
        let mut scanner = Scanner::new(source);
        let (tokens, lexical_error) = scanner.scan_tokens();

        let mut parser = Parser::new(tokens);
        let statements = parser.parse()?;

        if lexical_error {
            return Err(Error::Lexical);
        }

        let mut resolver = Resolver::new(&mut self.interpreter);
        resolver.resolve(&statements);

        if resolver.had_error {
            return Err(Error::Semantic);
        }

        self.interpreter.interpret(&statements)?;

        Ok(())
    }

    pub fn run_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let source = fs::read_to_string(file_path)?;
        if let Err(error) = self.run(&source) {
            match error {
                Error::Runtime { .. } => process::exit(70),
                _ => process::exit(65),
            }
        }

        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

            self.run(&line).ok();
            line.clear();
        }

        Ok(())
    }
}
