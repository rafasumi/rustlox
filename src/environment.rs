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

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment = self
            .enclosing
            .clone()
            .expect(&format!("No ancestor at distance {}.", 1));

        for i in 1..distance {
            let ancestor = environment
                .borrow()
                .enclosing
                .clone()
                .expect(&format!("No ancestor at distance {}.", i + 1));
            environment = ancestor.clone();
        }

        environment
    }

    pub fn get_at(&self, distance: usize, name: &Token) -> Result<Object, Error> {
        if distance == 0 {
            self.get(name)
        } else {
            // We don't expect this to panic,
            // because the Resolver already found the scope of the variable
            self.ancestor(distance).borrow().get(name)
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Object) -> Result<(), Error> {
        if distance == 0 {
            self.assign(name, value)
        } else {
            self.ancestor(distance).borrow_mut().assign(name, value)
        }
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
