use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{AstVisitor, Expr, Object, Stmt};
use crate::environment::Environment;
use crate::error::{runtime_error, Error};
use crate::token::{Token, TokenType};

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Rc::new(RefCell::new(Environment::new_global())),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for statement in statements {
            if let Err(e) = self.visit_stmt(statement) {
                runtime_error(&e);
                return Err(e);
            }
        }

        Ok(())
    }

    fn is_truthy(object: &Object) -> bool {
        match object {
            Object::Nil => false,
            Object::Boolean(value) => *value,
            _ => true,
        }
    }

    fn number_operand_err(operator: &Token) -> Result<Object, Error> {
        Err(Error::Runtime {
            token: operator.to_owned(),
            message: String::from("Operands must be numbers."),
        })
    }

    fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Error> {
        // This assignment here is the main reason why 'environment' is encapsulated in
        // Rc and RefCell smart pointers.
        let previous = self.environment.clone();

        // We use this IIFE because we want to reassign 'previous' to 'self.environment'
        // even if there are errors, but Rust error handling doesn't work like Java's,
        // that has a try-finally syntax.
        let result = || -> Result<(), Error> {
            self.environment = environment;

            for statement in statements {
                self.visit_stmt(statement)?;
            }

            Ok(())
        }();

        self.environment = previous;

        result
    }
}

impl AstVisitor<Result<Object, Error>, Result<(), Error>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Object, Error> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Grouping(expression) => self.visit_expr(expression),
            Expr::Unary { operator, right } => {
                let right = self.visit_expr(right)?;

                match operator.token_type {
                    TokenType::Minus => {
                        if let Object::Number(value) = right {
                            Ok(Object::Number(-value))
                        } else {
                            Interpreter::number_operand_err(operator)
                        }
                    }
                    TokenType::Bang => Ok(Object::Boolean(!Interpreter::is_truthy(&right))),
                    _ => unreachable!(),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.visit_expr(left)?;
                let right = self.visit_expr(right)?;

                match operator.token_type {
                    TokenType::Minus => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => Ok(Object::Number(lhs - rhs)),
                        _ => Interpreter::number_operand_err(operator),
                    },
                    TokenType::Plus => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => Ok(Object::Number(lhs + rhs)),
                        (Object::String(lhs), Object::String(rhs)) => {
                            Ok(Object::String(format!("{}{}", lhs, rhs)))
                        }
                        _ => Err(Error::Runtime {
                            token: operator.to_owned(),
                            message: String::from("Operands must be two numbers or two strings."),
                        }),
                    },
                    TokenType::Slash => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => Ok(Object::Number(lhs / rhs)),
                        _ => Interpreter::number_operand_err(operator),
                    },
                    TokenType::Star => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => Ok(Object::Number(lhs * rhs)),
                        _ => Interpreter::number_operand_err(operator),
                    },
                    TokenType::Percent => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => Ok(Object::Number(lhs % rhs)),
                        _ => Interpreter::number_operand_err(operator),
                    },
                    TokenType::Greater => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => {
                            Ok(Object::Boolean(lhs > rhs))
                        }
                        _ => Interpreter::number_operand_err(operator),
                    },
                    TokenType::GreaterEqual => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => {
                            Ok(Object::Boolean(lhs >= rhs))
                        }
                        _ => Interpreter::number_operand_err(operator),
                    },
                    TokenType::Less => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => {
                            Ok(Object::Boolean(lhs < rhs))
                        }
                        _ => Interpreter::number_operand_err(operator),
                    },
                    TokenType::LessEqual => match (left, right) {
                        (Object::Number(lhs), Object::Number(rhs)) => {
                            Ok(Object::Boolean(lhs <= rhs))
                        }
                        _ => Interpreter::number_operand_err(operator),
                    },
                    TokenType::BangEqual => Ok(Object::Boolean(!left.equals(&right))),
                    TokenType::EqualEqual => Ok(Object::Boolean(left.equals(&right))),
                    _ => unreachable!(),
                }
            }
            Expr::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.visit_expr(&condition)?;
                let then_val = self.visit_expr(&then_branch)?;
                let else_val = self.visit_expr(&else_branch)?;

                Ok(if Interpreter::is_truthy(&cond_val) {
                    then_val
                } else {
                    else_val
                })
            }
            Expr::Variable(name) => self.environment.borrow().get(name),
            Expr::Assign { name, value } => {
                let value = self.visit_expr(value)?;
                self.environment.borrow_mut().assign(name, value.clone())?;
                Ok(value)
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &crate::ast::Stmt) -> Result<(), Error> {
        match stmt {
            Stmt::Expression(expression) => {
                self.visit_expr(expression)?;
                Ok(())
            }
            Stmt::Print(expression) => {
                let value = self.visit_expr(expression)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(expr) = initializer {
                    self.visit_expr(expr)?
                } else {
                    Object::Nil
                };

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), value);

                Ok(())
            }
            Stmt::Block(statements) => {
                self.execute_block(
                    statements,
                    Rc::new(RefCell::new(Environment::new_local(
                        self.environment.clone(),
                    ))),
                )?;
                Ok(())
            }
        }
    }
}
