use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ast::{AstVisitor, Expr, Object, Stmt};
use crate::callable::LoxCallable;
use crate::class::LoxClass;
use crate::environment::Environment;
use crate::error::{runtime_error, Error};
use crate::token::{Token, TokenType};

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Token, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new_global()));

        globals.borrow_mut().define(
            String::from("clock"),
            Object::Callable(LoxCallable::LoxNative {
                call_impl: |_| -> Object {
                    Object::Number(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap() // Can safely unwrap here because SystemTime::now() will not be before EPOCH
                            .as_micros() as f64,
                    )
                },
                arity: 0,
            }),
        );

        Self {
            globals: globals.clone(),
            environment: globals.clone(),
            locals: HashMap::new(),
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

    pub fn execute_block(
        &mut self,
        statements: &Vec<Stmt>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Error> {
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

    pub fn resolve(&mut self, name: Token, depth: usize) {
        self.locals.insert(name, depth);
    }

    fn look_up_variable(&self, name: &Token) -> Result<Object, Error> {
        if let Some(distance) = self.locals.get(name) {
            self.environment.borrow().get_at(*distance, &name.lexeme)
        } else {
            self.globals.borrow().get(name)
        }
    }
}

impl AstVisitor<Result<Object, Error>, Result<(), Error>> for Interpreter {
    fn visit_expr(&mut self, expr: &Expr) -> Result<Object, Error> {
        match expr {
            Expr::Literal(value) => Ok(value.to_owned()),
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

                Ok(if Interpreter::is_truthy(&cond_val) {
                    self.visit_expr(then_branch)?
                } else {
                    self.visit_expr(else_branch)?
                })
            }
            Expr::Variable(name) => self.look_up_variable(name),
            Expr::Assign { name, value } => {
                let value = self.visit_expr(value)?;

                if let Some(distance) = self.locals.get(name) {
                    self.environment
                        .borrow_mut()
                        .assign_at(*distance, name, value.clone())?;
                } else {
                    self.globals.borrow_mut().assign(name, value.clone())?;
                }

                Ok(value)
            }
            Expr::Lambda { .. } => Ok(Object::Callable(LoxCallable::LoxFunction {
                name: None,
                definition: Box::new(expr.to_owned()),
                closure: self.environment.clone(),
                is_initializer: false,
            })),
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.visit_expr(&left)?;
                if operator.token_type == TokenType::Or {
                    if Interpreter::is_truthy(&left) {
                        return Ok(left);
                    }
                } else {
                    if !Interpreter::is_truthy(&left) {
                        return Ok(left);
                    }
                }

                Ok(self.visit_expr(&right)?)
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.visit_expr(callee)?;

                let mut evaluated_arguments = Vec::new();
                for argument in arguments {
                    evaluated_arguments.push(self.visit_expr(argument)?);
                }

                if let Object::Callable(function) = callee {
                    if evaluated_arguments.len() == function.arity() {
                        Ok(function.call(self, &evaluated_arguments)?)
                    } else {
                        Err(Error::Runtime {
                            token: paren.to_owned(),
                            message: format!(
                                "Expected {} arguments but got {}.",
                                function.arity(),
                                evaluated_arguments.len()
                            ),
                        })
                    }
                } else {
                    Err(Error::Runtime {
                        token: paren.to_owned(),
                        message: String::from("Can only call functions and classes."),
                    })
                }
            }
            Expr::Get { object, name } => {
                if let Object::Instance(instance) = self.visit_expr(&object)? {
                    instance.borrow().get(name, &instance)
                } else {
                    Err(Error::Runtime {
                        token: name.to_owned(),
                        message: String::from("Only instances have properties."),
                    })
                }
            }
            Expr::Set {
                object,
                name,
                value,
            } => {
                if let Object::Instance(instance) = self.visit_expr(&object)? {
                    let value = self.visit_expr(&value)?;
                    instance
                        .borrow_mut()
                        .set(name.lexeme.clone(), value.clone());
                    Ok(value)
                } else {
                    Err(Error::Runtime {
                        token: name.to_owned(),
                        message: String::from("Only instances have fields."),
                    })
                }
            }
            Expr::This(keyword) => self.look_up_variable(keyword),
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), Error> {
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
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition = self.visit_expr(condition)?;
                if Interpreter::is_truthy(&condition) {
                    self.visit_stmt(then_branch)?;
                } else if let Some(statement) = else_branch {
                    self.visit_stmt(statement)?;
                }

                Ok(())
            }
            Stmt::While { condition, body } => {
                while Interpreter::is_truthy(&self.visit_expr(condition)?) {
                    self.visit_stmt(body)?;
                }

                Ok(())
            }
            Stmt::Function { name, definition } => {
                let function = LoxCallable::LoxFunction {
                    name: Some(name.to_owned()),
                    definition: Box::new(definition.to_owned()),
                    closure: self.environment.clone(),
                    is_initializer: false,
                };

                self.environment
                    .borrow_mut()
                    .define(name.lexeme.to_owned(), Object::Callable(function));

                Ok(())
            }
            Stmt::Return { value, .. } => {
                let value = if let Some(return_value) = value {
                    self.visit_expr(return_value)?
                } else {
                    Object::Nil
                };

                Err(Error::Return(value))
            }
            Stmt::Class { name, methods } => {
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.clone(), Object::Nil);

                let mut method_map: HashMap<String, LoxCallable> = HashMap::new();
                for method in methods {
                    if let Stmt::Function { name, definition } = method {
                        let func = LoxCallable::LoxFunction {
                            name: Some(name.to_owned()),
                            definition: Box::new(definition.to_owned()),
                            closure: self.environment.clone(),
                            is_initializer: name.lexeme.eq("init"),
                        };
                        method_map.insert(name.lexeme.to_owned(), func);
                    }
                }

                self.environment.borrow_mut().assign(
                    name,
                    Object::Callable(LoxCallable::LoxClass {
                        class: Rc::new(LoxClass::new(name.lexeme.clone(), method_map)),
                    }),
                )?;

                Ok(())
            }
        }
    }
}
