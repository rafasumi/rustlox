mod error;
mod scanner;
mod token;

use scanner::Scanner;
use std::{
    error::Error,
    fs,
    io::{self, Write},
};

// TO-DO: implement behavior of not running code when there are errors 

fn run(source: &str) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();

    for token in tokens {
        println!("{token}");
    }

    Ok(())
}

pub fn run_file(file_path: &str) -> Result<(), Box<dyn Error>> {
    let source = fs::read_to_string(file_path)?;
    run(&source)?;

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

        run(&line)?;
        line.clear();
    }

    Ok(())
}
