use crate::ast::*;
use crate::error::parse_error;
use crate::token::*;

// Used macro to implement the "match" method because Rust functions can't be
// variadic
macro_rules! match_types {
    ($self:ident, $($token_type:expr),* ) => {
        if $($self.check($token_type)) ||* {
            $self.advance();
            true
        } else {
            false
        }
    };
}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expr, ()> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, ()> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ()> {
        let mut expr = self.comparison()?;

        while match_types!(self, TokenType::BangEqual, TokenType::EqualEqual) {
            let operator = self.previous().to_owned();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ()> {
        let mut expr = self.term()?;

        while match_types!(
            self,
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual
        ) {
            let operator = self.previous().to_owned();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ()> {
        let mut expr = self.factor()?;

        while match_types!(self, TokenType::Minus, TokenType::Plus) {
            let operator = self.previous().to_owned();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ()> {
        let mut expr = self.unary()?;

        while match_types!(self, TokenType::Slash, TokenType::Star, TokenType::Percent) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ()> {
        if match_types!(self, TokenType::Bang, TokenType::Minus) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, ()> {
        let expr = match &self.peek().token_type {
            TokenType::False => Expr::Literal(LiteralValue::Boolean(false)),
            TokenType::True => Expr::Literal(LiteralValue::Boolean(true)),
            TokenType::Nil => Expr::Literal(LiteralValue::Nil),
            TokenType::Number(literal) => Expr::Literal(LiteralValue::Number(literal.to_owned())),
            TokenType::String(literal) => Expr::Literal(LiteralValue::String(literal.to_owned())),
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
                Expr::Grouping(Box::new(expr))
            }
            _ => {
                parse_error(self.peek(), "Expect expression.");
                return Err(());
            }
        };

        self.advance();
        Ok(expr)
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, ()> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            parse_error(self.previous(), message);
            Err(())
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => self.advance(),
            };
        }
    }
}
