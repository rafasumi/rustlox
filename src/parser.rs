use crate::ast::*;
use crate::error::{error_token, Error};
use crate::token::*;

// Used a macro to implement the "match" method because Rust functions can't be
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();
        let mut had_error = false;
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(_) => {
                    had_error = true;
                    self.synchronize();
                }
            }
        }

        if !had_error {
            Ok(statements)
        } else {
            Err(Error::Syntax)
        }
    }

    fn expression(&mut self) -> Result<Expr, ()> {
        self.assignment()
    }

    fn declaration(&mut self) -> Result<Stmt, ()> {
        if match_types!(self, TokenType::Var) {
            self.var_declaration()
        } else if self.check(TokenType::Fun) && self.check_next(TokenType::Identifier) {
            self.advance();
            self.function("function")
        } else if match_types!(self, TokenType::Class) {
            self.class_declaration()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Result<Stmt, ()> {
        if match_types!(self, TokenType::If) {
            self.if_statement()
        } else if match_types!(self, TokenType::Print) {
            self.print_statement()
        } else if match_types!(self, TokenType::Return) {
            self.return_statement()
        } else if match_types!(self, TokenType::For) {
            self.for_statement()
        } else if match_types!(self, TokenType::While) {
            self.while_statement()
        } else if match_types!(self, TokenType::LeftBrace) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ()> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if match_types!(self, TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn print_statement(&mut self) -> Result<Stmt, ()> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value")?;
        Ok(Stmt::Print(value))
    }

    fn return_statement(&mut self) -> Result<Stmt, ()> {
        let keyword = self.previous().to_owned();

        let value = if !self.check(TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expect ';' after value.")?;

        Ok(Stmt::Return { keyword, value })
    }

    fn for_statement(&mut self) -> Result<Stmt, ()> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if match_types!(self, TokenType::Semicolon) {
            None
        } else if match_types!(self, TokenType::Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Object::Boolean(true))
        };

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = if !self.check(TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(inc_expr) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc_expr)]);
        }

        // Desugaring
        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(init_stmt) = initializer {
            body = Stmt::Block(vec![init_stmt, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ()> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;

        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ()> {
        let name = self
            .consume(TokenType::Identifier, "Expect variable name.")?
            .to_owned();
        let initializer = if match_types!(self, TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, ()> {
        let name = self
            .consume(TokenType::Identifier, &format!("Expect {kind} name."))?
            .to_owned();

        Ok(Stmt::Function {
            name,
            definition: self.function_expr(kind)?,
        })
    }

    fn class_declaration(&mut self) -> Result<Stmt, ()> {
        let name = self
            .consume(TokenType::Identifier, "Expect class name.")?
            .to_owned();

        let superclass = if match_types!(self, TokenType::Less) {
            self.consume(TokenType::Identifier, "Expect superclass name")?;
            Some(Expr::Variable(self.previous().to_owned()))
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expect '{' before class body.")?;

        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after class body.")?;

        Ok(Stmt::Class {
            name,
            superclass,
            methods,
        })
    }

    fn expression_statement(&mut self) -> Result<Stmt, ()> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression")?;
        Ok(Stmt::Expression(value))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ()> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?)
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn assignment(&mut self) -> Result<Expr, ()> {
        let expr = self.ternary()?;

        if match_types!(self, TokenType::Equal) {
            let equals = self.previous().to_owned();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            } else if let Expr::Get { object, name } = expr {
                return Ok(Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                });
            }

            error_token(&equals, "Invalid assignment target.");
        }

        Ok(expr)
    }

    fn ternary(&mut self) -> Result<Expr, ()> {
        let mut expr = self.or()?;

        if match_types!(self, TokenType::Question) {
            let then_branch = self.ternary()?;

            if !self.check(TokenType::Colon) {
                error_token(self.previous(), "Expect ':' in ternary expression");
                return Err(());
            }

            self.advance();
            let else_branch = self.ternary()?;

            expr = Expr::Ternary {
                condition: Box::new(expr),
                then_branch: Box::new(then_branch),
                else_branch: Box::new(else_branch),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ()> {
        let mut expr = self.and()?;

        while match_types!(self, TokenType::Or) {
            let operator = self.previous().to_owned();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ()> {
        let mut expr = self.equality()?;

        while match_types!(self, TokenType::And) {
            let operator = self.previous().to_owned();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
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
            self.call()
        }
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ()> {
        let mut arguments = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    error_token(self.peek(), "Can't have more than 255 arguments.");
                }

                arguments.push(self.expression()?);

                if !match_types!(self, TokenType::Comma) {
                    break;
                }
            }
        }

        let paren = self
            .consume(TokenType::RightParen, "Expect ')' after arguments")?
            .to_owned();

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn call(&mut self) -> Result<Expr, ()> {
        let mut expr = self.primary()?;

        loop {
            if match_types!(self, TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if match_types!(self, TokenType::Dot) {
                let name = self
                    .consume(TokenType::Identifier, "Expect property name after '.'.")?
                    .to_owned();
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn function_expr(&mut self, kind: &str) -> Result<Expr, ()> {
        self.consume(TokenType::LeftParen, &format!("Expect '('."))?;

        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    error_token(self.peek(), "Can't have more than 255 parameters.");
                }

                params.push(
                    self.consume(TokenType::Identifier, "Expect parameter name.")?
                        .to_owned(),
                );

                if !match_types!(self, TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.")?;

        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {kind} body."),
        )?;
        let body = self.block()?;

        Ok(Expr::Lambda { params, body })
    }

    fn primary(&mut self) -> Result<Expr, ()> {
        let expr = match &self.peek().token_type {
            TokenType::False => Expr::Literal(Object::Boolean(false)),
            TokenType::True => Expr::Literal(Object::Boolean(true)),
            TokenType::Nil => Expr::Literal(Object::Nil),
            TokenType::Number(literal) => Expr::Literal(Object::Number(literal.to_owned())),
            TokenType::String(literal) => Expr::Literal(Object::String(literal.to_owned())),
            TokenType::Identifier => Expr::Variable(self.peek().to_owned()),
            TokenType::Fun => {
                self.advance();
                return Ok(self.function_expr("function")?);
            }
            TokenType::LeftParen => {
                // This is needed to consume the LeftParen Token, since we don't use match_types! here
                self.advance();
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expect ')' after expression.")?;
                return Ok(Expr::Grouping(Box::new(expr)));
            }
            TokenType::Super => {
                let keyword = self.peek().to_owned();
                // This is needed to consume the Super Token, since we don't use match_types! here
                self.advance();

                self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;
                let method = self
                    .consume(TokenType::Identifier, "Expect superclass method name.")?
                    .to_owned();
                return Ok(Expr::Super { keyword, method });
            }
            TokenType::This => Expr::This(self.peek().to_owned()),
            _ => {
                error_token(self.peek(), "Expect expression.");
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
            error_token(self.peek(), message);
            Err(())
        }
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn check_next(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else if self.tokens[self.current + 1].token_type == TokenType::EOF {
            false
        } else {
            self.tokens[self.current + 1].token_type == token_type
        }
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
