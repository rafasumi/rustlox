use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::ast::{Expr, Object};
use crate::class::{LoxClass, LoxInstance};
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
        is_initializer: bool,
    },
    LoxClass {
        class: Rc<LoxClass>,
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
                is_initializer,
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
                        Ok(_) => {
                            if *is_initializer {
                                closure.borrow().get_at(0, "this")
                            } else {
                                Ok(Object::Nil)
                            }
                        }
                        Err(Error::Return(value)) => {
                            if *is_initializer {
                                closure.borrow().get_at(0, "this")
                            } else {
                                Ok(value)
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
                _ => unreachable!(),
            },
            LoxCallable::LoxClass { class } => {
                let instance = Rc::new(RefCell::new(LoxInstance::new(class.clone())));

                if let Some(initializer) = class.find_method(&String::from("init")) {
                    initializer
                        .bind(Object::Instance(instance.clone()))
                        .call(interpreter, arguments)?;
                }

                Ok(Object::Instance(instance))
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::LoxNative { arity, .. } => *arity,
            LoxCallable::LoxFunction { definition, .. } => match definition.as_ref() {
                Expr::Lambda { params, .. } => params.len(),
                _ => unreachable!(),
            },
            LoxCallable::LoxClass { class } => {
                if let Some(initializer) = class.find_method(&String::from("init")) {
                    initializer.arity()
                } else {
                    0
                }
            }
        }
    }

    pub fn bind(&self, instance: Object) -> LoxCallable {
        match self {
            LoxCallable::LoxFunction {
                name,
                definition,
                closure,
                is_initializer,
            } => {
                let mut env = Environment::new_local(closure.clone());
                env.define(String::from("this"), instance);
                LoxCallable::LoxFunction {
                    name: name.to_owned(),
                    definition: definition.to_owned(),
                    closure: Rc::new(RefCell::new(env)),
                    is_initializer: is_initializer.to_owned(),
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn equals(&self, other: &LoxCallable) -> bool {
        // TODO: add more equalities for LoxCallable
        match (self, other) {
            (
                LoxCallable::LoxClass { class: class_self },
                LoxCallable::LoxClass { class: class_other },
            ) => Rc::ptr_eq(class_self, class_other),
            _ => false,
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
            LoxCallable::LoxClass { class } => write!(f, "{}", class.to_string()),
        }
    }
}
