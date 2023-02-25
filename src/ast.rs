use std::fmt;

use crate::token::Token;

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
}

pub enum Stmt {
    Expression(Expr),
    Print(Expr),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block(Vec<Stmt>),
}

#[derive(Clone)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Object {
    pub fn equals(&self, other: &Object) -> bool {
        match (self, other) {
            (Object::Boolean(lhs), Object::Boolean(rhs)) => lhs == rhs,
            (Object::Number(lhs), Object::Number(rhs)) => lhs == rhs,
            (Object::String(lhs), Object::String(rhs)) => lhs == rhs,
            (Object::Nil, Object::Nil) => true,
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
        }
    }
}

pub trait AstVisitor<T, U> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_stmt(&mut self, stmt: &Stmt) -> U;
}
