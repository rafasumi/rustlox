use itertools::{Itertools, MultiPeek};
use phf_macros::phf_map;
use std::str::Chars;

use crate::error::error_line;
use crate::token::{Token, TokenType};

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "for" => TokenType::For,
    "fun" => TokenType::Fun,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While
};

pub struct Scanner<'a> {
    source: String,
    source_iter: MultiPeek<Chars<'a>>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.to_owned(),
            source_iter: source.chars().multipeek(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> (&Vec<Token>, bool) {
        let mut had_error = false;
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            if let Err(_) = self.scan_token() {
                had_error = true;
            }
        }

        self.tokens.push(Token::new(TokenType::EOF, "", self.line, self.current.clone()));
        (&self.tokens, had_error)
    }

    fn scan_token(&mut self) -> Result<(), ()> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '%' => self.add_token(TokenType::Percent),
            '?' => self.add_token(TokenType::Question),
            ':' => self.add_token(TokenType::Colon),
            '!' => {
                let token_type = if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(token_type);
            }
            '/' => {
                if self.match_next('/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.match_next('*') {
                    self.block_comment()?;
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string()?,
            c => {
                if c.is_digit(10) {
                    self.number();
                } else if Scanner::is_alpha(c) {
                    self.identifier();
                } else {
                    error_line(&self.line, &format!("Unexpected character: \"{c}\"."));
                    return Err(());
                }
            }
        };
        Ok(())
    }

    fn identifier(&mut self) {
        while Scanner::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = KEYWORDS.get(text).cloned().unwrap_or(TokenType::Identifier);
        self.add_token(token_type);
    }

    fn number(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // Consume the "."
            self.advance();

            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let literal = self.source[self.start..self.current]
            .parse::<f64>()
            .expect("Unable to parse number.");
        self.add_token(TokenType::Number(literal));
    }

    fn string(&mut self) -> Result<(), ()> {
        while !self.is_at_end() {
            let peek = self.peek();
            if peek == '"' {
                break;
            }

            if peek == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            error_line(&self.line, "Unterminated string.");
            return Err(());
        }

        // The closing double quotation mark.
        self.advance();

        // Trim the surrounding quotes.
        let literal = self.source[self.start + 1..self.current - 1].to_owned();
        self.add_token(TokenType::String(literal));
        Ok(())
    }

    fn block_comment(&mut self) -> Result<(), ()> {
        let mut comment_level = 1;
        while !self.is_at_end() {
            let peek = self.peek();
            let peek_next = self.peek_next();

            if peek == '/' && peek_next == '*' {
                comment_level += 1;
            }

            if peek == '*' && peek_next == '/' {
                comment_level -= 1;
            }

            if comment_level == 0 {
                self.advance();
                self.advance();
                break;
            }

            let c = self.advance();
            if c == '\n' {
                self.line += 1;
            }
        }

        if comment_level != 0 {
            error_line(&self.line, "Unterminated block comment.");
            return Err(());
        }

        Ok(())
    }

    fn peek(&mut self) -> char {
        self.source_iter.reset_peek();
        *self.source_iter.peek().unwrap_or(&'\0')
    }

    fn peek_next(&mut self) -> char {
        self.source_iter.reset_peek();
        self.source_iter.peek(); // Advance peeking "cursor"
        *self.source_iter.peek().unwrap_or(&'\0')
    }

    fn is_alpha(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme = &self.source[self.start..self.current];
        self.tokens.push(Token::new(token_type, lexeme, self.line, self.current.clone()))
    }

    fn advance(&mut self) -> char {
        let next_char = self.source_iter.next().expect("Unexpected end.");
        // This is needed because Rust characters can use more than one byte.
        self.current += next_char.len_utf8();

        next_char
    }

    fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.peek() != expected {
            return false;
        }

        self.advance();
        true
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
}
