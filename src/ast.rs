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
    Literal(LiteralValue),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
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
                LiteralValue::String(string_val) => string_val.to_owned(),
                LiteralValue::Number(number_val) => number_val.to_string(),
                LiteralValue::Boolean(bool_val) => bool_val.to_string(),
                LiteralValue::Nil => String::from("nil"),
            },
            Expr::Unary { operator, right } => self.parenthesize(&operator.lexeme, vec![&right]),
        }
    }
}
