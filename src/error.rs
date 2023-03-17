use crate::ast::Object;
use crate::token::{Token, TokenType};

pub enum Error {
    Lexical,
    Syntax,
    Semantic,
    Runtime { token: Token, message: String },
    Return(Object), // Used to interrupt execution flow and propagate return value
}

pub fn error_line(line: &u32, message: &str) {
    report(line, "", message);
}

pub fn error_token(token: &Token, message: &str) {
    if token.token_type == TokenType::EOF {
        report(&token.line, " at end", message);
    } else {
        report(&token.line, &format!(" at '{}'", token.lexeme), message);
    }
}

pub fn runtime_error(error: &Error) {
    if let Error::Runtime { token, message } = error {
        eprintln!("[line {}] {}", token.line, message);
    }
}

pub fn report(line: &u32, location: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, location, message);
}
