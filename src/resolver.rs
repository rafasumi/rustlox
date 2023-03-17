use std::collections::HashMap;

use crate::ast::{AstVisitor, Expr, Stmt};
use crate::error::error_token;
use crate::interpreter::Interpreter;
use crate::token::Token;

#[derive(PartialEq)]
enum VarState {
    Declared,
    Defined,
    Used,
}

struct Var {
    name: Token,
    state: VarState,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, Var>>,
    current_function: FunctionType,
    pub had_error: bool,
}

#[derive(Clone, PartialEq)]
enum FunctionType {
    None,
    Function,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            had_error: false,
        }
    }

    pub fn resolve(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.visit_stmt(statement);
        }
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<Stmt>, func_type: FunctionType) {
        let enclosing_function = self.current_function.clone();
        self.current_function = func_type;

        self.begin_scope();

        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve(body);

        self.end_scope();
        self.current_function = enclosing_function;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            for var in scope.values() {
                if var.state != VarState::Used {
                    self.error(
                        &var.name,
                        &format!("Variable '{}' is never used.", var.name.lexeme),
                    );
                }
            }
        }
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            let had_key = scope.contains_key(&name.lexeme);
            scope.insert(
                name.lexeme.clone(),
                Var {
                    name: name.to_owned(),
                    state: VarState::Declared,
                },
            );

            if had_key {
                self.error(&name, "Already a variable with this name in this scope.");
            }
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(var) = scope.get_mut(&name.lexeme) {
                var.state = VarState::Defined;
            }
        }
    }

    fn resolve_local(&mut self, name: &Token, is_used: bool) {
        for (index, scope) in self.scopes.iter_mut().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(name.clone(), index.clone());

                if is_used {
                    scope.get_mut(&name.lexeme).unwrap().state = VarState::Used;
                }

                return;
            }
        }
    }

    fn error(&mut self, token: &Token, message: &str) {
        error_token(token, message);
        self.had_error = true;
    }
}

impl<'a> AstVisitor<(), ()> for Resolver<'a> {
    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Variable(name) => {
                if let Some(scope) = self.scopes.last() {
                    if let Some(var) = scope.get(&name.lexeme) {
                        if var.state == VarState::Declared {
                            self.error(&name, "Can't read local variable in its own initializer.");
                        }
                    }
                }

                self.resolve_local(name, true);
            }
            Expr::Assign { name, value } => {
                self.visit_expr(value);
                self.resolve_local(name, false);
            }
            Expr::Lambda { params, body } => {
                self.resolve_function(params, body, FunctionType::Function);
            }
            Expr::Ternary {
                condition,
                then_branch,
                else_branch,
            } => {
                self.visit_expr(&condition);
                self.visit_expr(&then_branch);
                self.visit_expr(&else_branch);
            }
            Expr::Binary { left, right, .. } => {
                self.visit_expr(&left);
                self.visit_expr(&right);
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.visit_expr(&callee);
                for argument in arguments {
                    self.visit_expr(argument);
                }
            }
            Expr::Grouping(expr) => self.visit_expr(expr),
            Expr::Logical { left, right, .. } => {
                self.visit_expr(&left);
                self.visit_expr(&right);
            }
            Expr::Unary { right, .. } => self.visit_expr(&right),
            Expr::Literal(_) => (),
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(statements) => {
                self.begin_scope();
                self.resolve(statements);
                self.end_scope();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(expr) = initializer {
                    self.visit_expr(expr);
                }
                self.define(name);
            }
            Stmt::Function { name, definition } => {
                self.declare(name);
                self.define(name);
                self.visit_expr(definition);
            }
            Stmt::Expression(expr) => self.visit_expr(expr),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.visit_expr(condition);
                self.visit_stmt(&then_branch);
                if let Some(else_stmt) = else_branch {
                    self.visit_stmt(&else_stmt);
                }
            }
            Stmt::Print(expr) => self.visit_expr(expr),
            Stmt::Return { keyword, value } => {
                if self.current_function == FunctionType::None {
                    self.error(&keyword, "Can't return from top-level code.");
                }

                if let Some(expression) = value {
                    self.visit_expr(expression);
                }
            }
            Stmt::While { condition, body } => {
                self.visit_expr(condition);
                self.visit_stmt(&body);
            }
        }
    }
}
