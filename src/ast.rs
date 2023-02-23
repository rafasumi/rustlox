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

pub trait Visitor<T> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
}

pub struct AstPrinter;

impl AstPrinter {
    fn parenthesize(&mut self, name: &str, exprs: Vec<&Expr>) -> String {
        let mut output = String::from("(");
        output.push_str(name);

        for expr in exprs {
            output.push_str(" ");
            output.push_str(&self.visit_expr(expr));
        }
        output.push_str(")");
        output
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                format!(
                    "({} ? {} : {})",
                    self.visit_expr(condition),
                    self.visit_expr(then_branch),
                    self.visit_expr(else_branch)
                )
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => self.parenthesize(&operator.lexeme, vec![&left, &right]),
            Expr::Grouping(expression) => self.parenthesize("group", vec![&expression]),
            Expr::Literal(value) => match value {
                Object::String(string_val) => string_val.to_owned(),
                Object::Number(number_val) => number_val.to_string(),
                Object::Boolean(bool_val) => bool_val.to_string(),
                Object::Nil => String::from("nil"),
            },
            Expr::Unary { operator, right } => self.parenthesize(&operator.lexeme, vec![&right]),
        }
    }
}
