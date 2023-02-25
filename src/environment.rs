use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::Object;
use crate::error::Error;
use crate::token::Token;

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub fn new_global() -> Environment {
        Environment {
            enclosing: None,
            values: HashMap::new(),
        }
    }

    pub fn new_local(enclosing: Rc<RefCell<Environment>>) -> Environment {
        Environment {
            enclosing: Some(enclosing),
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Object, Error> {
        if let Some(value) = self.values.get(&name.lexeme) {
            Ok(value.to_owned())
        } else {
            if let Some(env) = &self.enclosing {
                env.borrow().get(name)
            } else {
                Err(Error::Runtime {
                    token: name.to_owned(),
                    message: format!("Undefined variable '{}'.", name.lexeme),
                })
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: Object) -> Result<(), Error> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            Ok(())
        } else {
            if let Some(env) = &self.enclosing {
                env.borrow_mut().assign(name, value)
            } else {
                Err(Error::Runtime {
                    token: name.to_owned(),
                    message: format!("Undefined variable '{}'.", name.lexeme),
                })
            }
        }
    }
}
