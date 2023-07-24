use std::collections::HashMap;
use std::mem::replace;

use crate::ast::{AstVisitor, Expr, Stmt};
use crate::error::error_token;
use crate::interpreter::Interpreter;
use crate::token::Token;

enum VarState {
    Declared,
    Defined,
    Used,
}

struct Var {
    name: Option<Token>,
    state: VarState,
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, Var>>,
    current_function: FunctionType,
    current_class: ClassType,
    pub had_error: bool,
}

enum FunctionType {
    None,
    Function,
    Initializer,
    Method,
}

enum ClassType {
    None,
    Class,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::None,
            current_class: ClassType::None,
            had_error: false,
        }
    }

    pub fn resolve(&mut self, statements: &Vec<Stmt>) {
        for statement in statements {
            self.visit_stmt(statement);
        }
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<Stmt>, func_type: FunctionType) {
        let enclosing_function = replace(&mut self.current_function, func_type);

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
                if let VarState::Used = var.state {
                    continue;
                }

                if let Some(name) = &var.name {
                    self.error(&name, &format!("Variable '{}' is never used.", name.lexeme));
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
                    name: Some(name.to_owned()),
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
                        if let VarState::Declared = var.state {
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
            Expr::Get { object, .. } => {
                self.visit_expr(&object);
            }
            Expr::Set { object, value, .. } => {
                self.visit_expr(&value);
                self.visit_expr(&object);
            }
            Expr::This(keyword) => {
                if let ClassType::None = self.current_class {
                    self.error(keyword, "Can't use 'this' outside of a class.")
                }

                self.resolve_local(keyword, true)
            }
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
                if let FunctionType::None = self.current_function {
                    self.error(&keyword, "Can't return from top-level code.");
                }

                if let Some(expression) = value {
                    if let FunctionType::Initializer = self.current_function {
                        self.error(&keyword, "Can't return a value from an initializer.")
                    }
                    self.visit_expr(expression);
                }
            }
            Stmt::While { condition, body } => {
                self.visit_expr(condition);
                self.visit_stmt(&body);
            }
            Stmt::Class { name, methods } => {
                let enclosing_class = replace(&mut self.current_class, ClassType::Class);

                self.declare(name);
                self.define(name);

                self.begin_scope();
                self.scopes.last_mut().unwrap().insert(
                    String::from("this"),
                    Var {
                        name: None,            // Doesn't have a name Token, as it's not declared
                        state: VarState::Used, // Assume that 'this' is always used
                    },
                );

                for method in methods {
                    if let Stmt::Function { definition, name } = method {
                        if let Expr::Lambda { params, body } = definition {
                            let func_type = if name.lexeme == "init" {
                                FunctionType::Initializer
                            } else {
                                FunctionType::Method
                            };

                            self.resolve_function(params, body, func_type);
                        }
                    }
                }

                self.end_scope();

                self.current_class = enclosing_class;
            }
        }
    }
}
