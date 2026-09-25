/*
 * evalator.rs
*/

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::parser::SExpr;


// ============================================================
// Value
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Atom(String),
    Nil,
    Cons(Rc<Value>, Rc<Value>>),

    Builtin(BuiltinFunction),
    Lambda(Rc<Closure>),
}


// ============================================================
// Builtin Function
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFunction {
    Atom,
    Eq,
    Car,
    Cdr,
    Cons,
}

impl BuiltinFunction {
    fn call(
        &self,
        args: &[Value],
    ) -> Result<Value, EvalError> {
        match self {
            BuiltinFunction::Atom => {
                // ...
            }

            BuiltinFunction::Eq => {
                // ...
            }

            BuiltinFunction::Car => {
                // ...
            }

            BuiltinFunction::Cdr => {
                // ...
            }

            BuiltinFunction::Cons => {
                // ...
            }
        }
    }
}


// ============================================================
// Closure
// ============================================================

pub struct Closure {
    params: Vec<String>,
    body: SExpr,
    env: Rc<RefCell<Environment>>,
}


// ============================================================
// Environment
// ============================================================

pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        // ...
    }

    pub fn with_parent(
        parent: Rc<RefCell<Environment>>,
    ) -> Self {
        // ...
    }

    pub fn define(
        &mut self,
        name: String,
        value: Value,
    ) {
        // ...
    }

    pub fn lookup(
        &self,
        name: &str,
    ) -> Option<Value> {
        // ...
    }
}


// ============================================================
// Eval Error
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    UndefinedSymbol(String),

    InvalidArgumentCount,

    InvalidArgumentType,

    NotCallable,

    InvalidSpecialForm,

    InvalidLambda,

    // 必要に応じて追加
}


// ============================================================
// Evaluator
// ============================================================

pub fn eval(
    expr: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {

    match expr {

        SExpr::Nil => {
            // ...
        }

        SExpr::Atom(name) => {
            // ...
        }

        SExpr::Cons(operator, arguments) => {
            // ...
        }
    }
}


// ============================================================
// Special Forms
// ============================================================

fn eval_special_form(
    name: &str,
    args: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {

    match name {

        "quote" => {
            // ...
        }

        "cond" => {
            // ...
        }

        "lambda" => {
            // ...
        }

        "define" => {
            // ...
        }

        _ => {
            // 特殊形式ではない
        }
    }
}


fn eval_quote(
    args: &SExpr,
) -> Result<Value, EvalError> {
    // ...
}


fn eval_cond(
    clauses: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {
    // ...
}


fn eval_lambda(
    args: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {
    // ...
}


fn eval_define(
    args: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {
    // ...
}


// ============================================================
// Function Application
// ============================================================

fn eval_application(
    operator: &SExpr,
    arguments: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {
    // ...
}


fn eval_arguments(
    args: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Vec<Value>, EvalError> {
    // ...
}


fn apply(
    function: Value,
    arguments: Vec<Value>,
) -> Result<Value, EvalError> {
    match function {

        Value::Builtin(builtin) => {
            // ...
        }

        Value::Lambda(closure) => {
            // ...
        }

        _ => {
            // ...
        }
    }
}


fn apply_lambda(
    closure: &Closure,
    arguments: Vec<Value>,
) -> Result<Value, EvalError> {
    // ...
}


// ============================================================
// Utility
// ============================================================

fn quote_to_value(
    expr: &SExpr,
) -> Value {
    // ...
}