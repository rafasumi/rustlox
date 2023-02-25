use std::{env, process};

use rustlox::RustLox;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut rustlox = RustLox::new();

    match args.as_slice() {
        [_, file_path] => {
            if let Err(err) = rustlox.run_file(file_path) {
                eprintln!("An error occurred: {err}");
                process::exit(1);
            }
        }
        [_] => {
            if let Err(err) = rustlox.run_prompt() {
                eprintln!("An error occurred: {err}");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("You can't pass more than one argument.");
            process::exit(64);
        }
    }
}
