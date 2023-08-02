use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::ast::Object;
use crate::callable::LoxCallable;
use crate::error::Error;
use crate::token::Token;

#[derive(Clone)]
pub struct LoxClass {
    pub name: String,
    superclass: Option<Rc<LoxClass>>,
    methods: HashMap<String, LoxCallable>,
}

#[derive(Clone)]
pub struct LoxInstance {
    class: Rc<LoxClass>,
    fields: HashMap<String, Object>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<Rc<LoxClass>>,
        methods: HashMap<String, LoxCallable>,
    ) -> Self {
        Self {
            name,
            superclass,
            methods,
        }
    }

    pub fn find_method(&self, name: &String) -> Option<&LoxCallable> {
        if self.methods.contains_key(name) {
            self.methods.get(name)
        } else if let Some(class) = &self.superclass {
            class.find_method(name)
        } else {
            None
        }
    }
}

impl LoxInstance {
    pub fn new(class: Rc<LoxClass>) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token, instance: &Rc<RefCell<LoxInstance>>) -> Result<Object, Error> {
        if let Some(field) = self.fields.get(&name.lexeme) {
            Ok(field.to_owned())
        } else if let Some(method) = self.class.find_method(&name.lexeme) {
            Ok(Object::Callable(
                method.bind(Object::Instance(instance.clone())),
            ))
        } else {
            Err(Error::Runtime {
                token: name.to_owned(),
                message: format!("Undefined property '{}'.", name.lexeme),
            })
        }
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.fields.insert(name, value);
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
