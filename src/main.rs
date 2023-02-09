use std::{env, process};

use rustlox::{run_file, run_prompt};

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_, file_path] => {
            if let Err(err) = run_file(file_path) {
                eprintln!("An error occurred: {err}");
                process::exit(1);
            }
        }
        [_] => {
            if let Err(err) = run_prompt() {
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
