use crate::token::{Token, TokenType};

pub enum Error {
    Lexical,
    Syntax,
    Runtime {
        token: Token,
        message: String
    }
}

pub fn error(line: &u32, message: &str) {
    report(line, "", message);
}

pub fn parse_error(token: &Token, message: &str) -> Error {
    if token.token_type == TokenType::EOF {
        report(&token.line, " at end", message);
    } else {
        report(&token.line, &format!(" at '{}'", token.lexeme), message);
    }

    Error::Syntax
}

pub fn runtime_error(error: &Error) {
    if let Error::Runtime { token, message } = error {
        eprintln!("[line {}] {}", token.line, message);
    }
}

pub fn report(line: &u32, location: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, location, message);
}
