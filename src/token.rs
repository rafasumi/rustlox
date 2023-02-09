use std::fmt;

pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: u32,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: &str, line: u32) -> Self {
        Self {
            token_type,
            lexeme: lexeme.to_owned(),
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.token_type {
            TokenType::String(literal) => {
                write!(f, "line {}: String {} {}", self.line, self.lexeme, literal)
            }
            TokenType::Number(literal) => {
                write!(f, "line {}: Number {} {}", self.line, self.lexeme, literal)
            }
            _ => write!(
                f,
                "line {}: {:?} {}",
                self.line, self.token_type, self.lexeme
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Slash,
    Modulo,
    Star,
    Semicolon,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    // String and number literals already have their runtime values in the TokenType
    String(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}
