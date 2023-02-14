use crate::token::{Token, TokenType};

pub fn error(line: &u32, message: &str) {
    report(line, "", message);
}

pub fn parse_error(token: &Token, message: &str) {
    if token.token_type == TokenType::EOF {
        report(&token.line, " at end", message);
    } else {
        report(&token.line, &format!(" at '{}'", token.lexeme), message);
    }
}

pub fn report(line: &u32, location: &str, message: &str) {
    eprintln!("[line {}] Error{}: {}", line, location, message);
}
