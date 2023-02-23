use crate::ast::{Expr, Object, Visitor};
use crate::error::{runtime_error, Error};
use crate::token::{Token, TokenType};

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn interpret(&mut self, expr: &Expr) -> Result<(), Error> {
        match self.visit_expr(expr) {
            Ok(val) => {
                println!("{val}");
                Ok(())
            }
            Err(e) => {
                runtime_error(&e);
                Err(e)
            }
        }
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
}

impl Visitor<Result<Object, Error>> for Interpreter {
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
                    TokenType::EqualEqual => Ok(Object::Boolean(!left.equals(&right))),
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
        }
    }
}
