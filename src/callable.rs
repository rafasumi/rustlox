use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::ast::{Expr, Object};
use crate::environment::Environment;
use crate::error::Error;
use crate::interpreter::Interpreter;
use crate::token::Token;

#[derive(Clone)]
pub enum LoxCallable {
    LoxNative {
        call_impl: fn(&Vec<Object>) -> Object,
        arity: usize,
    },
    LoxFunction {
        name: Option<Token>,
        definition: Box<Expr>,
        closure: Rc<RefCell<Environment>>,
    },
}

impl LoxCallable {
    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: &Vec<Object>,
    ) -> Result<Object, Error> {
        match self {
            LoxCallable::LoxNative { call_impl, .. } => Ok((call_impl)(arguments)),
            LoxCallable::LoxFunction {
                definition,
                closure,
                ..
            } => match definition.as_ref() {
                Expr::Lambda { params, body } => {
                    let environment =
                        Rc::new(RefCell::new(Environment::new_local(closure.clone())));

                    for (param, argument) in params.iter().zip(arguments) {
                        environment
                            .borrow_mut()
                            .define(param.lexeme.clone(), argument.clone())
                    }

                    match interpreter.execute_block(body, environment) {
                        Ok(_) => Ok(Object::Nil),
                        Err(Error::Return(value)) => Ok(value),
                        Err(e) => Err(e),
                    }
                }
                _ => unreachable!(),
            },
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::LoxNative { arity, .. } => *arity,
            LoxCallable::LoxFunction { definition, .. } => match definition.as_ref() {
                Expr::Lambda { params, .. } => params.len(),
                _ => unreachable!(),
            },
        }
    }
}

impl fmt::Display for LoxCallable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxCallable::LoxNative { .. } => write!(f, "<native fn>"),
            LoxCallable::LoxFunction { name, .. } => match name {
                Some(func_name) => write!(f, "<fn {}>", func_name.lexeme),
                None => write!(f, "<fn>"),
            },
        }
    }
}
