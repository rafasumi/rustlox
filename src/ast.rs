use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::callable::LoxCallable;
use crate::class::LoxInstance;
use crate::token::Token;

#[derive(Clone)]
pub enum Expr {
    Ternary {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Object),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable(Token),
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    This(Token),
    Lambda {
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
}

#[derive(Clone)]
pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: Token,
        definition: Expr,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Class {
        name: Token,
        methods: Vec<Stmt>,
    },
}

#[derive(Clone)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
    Callable(LoxCallable),
    Instance(Rc<RefCell<LoxInstance>>),
}

impl Object {
    pub fn equals(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::Boolean(lhs), Object::Boolean(rhs)) => lhs == rhs,
            (Object::Number(lhs), Object::Number(rhs)) => lhs == rhs,
            (Object::String(lhs), Object::String(rhs)) => lhs == rhs,
            (Object::Nil, Object::Nil) => true,
            (Object::Callable(lhs), Object::Callable(rhs)) => lhs.equals(rhs),
            _ => false,
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::String(val) => write!(f, "{}", val.to_string()),
            Object::Number(val) => write!(f, "{}", val.to_string()),
            Object::Boolean(val) => write!(f, "{}", val.to_string()),
            Object::Nil => write!(f, "nil"),
            Object::Callable(val) => write!(f, "{}", val.to_string()),
            Object::Instance(val) => write!(f, "{}", val.borrow().to_string()),
        }
    }
}

pub trait AstVisitor<T, U> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_stmt(&mut self, stmt: &Stmt) -> U;
}
