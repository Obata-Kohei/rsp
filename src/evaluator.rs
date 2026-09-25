/*
 * evaluator.rs
*/

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::{evaluator::Value::Builtin, parser::SExpr};


// 評価する値の種類
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Atom(String),
    Nil,
    Cons(Rc<Value>, Rc<Value>),

    Builtin(Builtin),
    Lambda(Rc<Closure>),
}


// ビルトイン関数
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinFunction {
    Atom,
    Eq,
    Car,
    Cdr,
    Cons,
}

impl BuiltinFunction {
    fn call(&self, args: &[Value]) -> Result<Value, EvalError> {
        match self {
            BuiltinFunction::Atom => {
                if args.len() != 1 {
                    return Err(EvalError::InvalidArgumentCount);
                }
                match args[0] {
                    Value::Atom(_) => Ok(Value::Atom("T".to_string())),
                    _ => Ok(Value::Nil),
                }
            }

            BuiltinFunction::Eq => {
                if args.len() != 2 {
                    return Err(EvalError::InvalidArgumentCount);
                }
                match (&args[0], &args[1]) {
                    (Value::Atom(a), Value::Atom(b)) if a == b => Ok(Value::Atom("T".to_string())),
                    (Value::Nil, value::Nil) => Ok(Value::Atom("T".to_string())),
                    _ => Ok(Value::Nil),
                }
            }

            BuiltinFunction::Car => {
                if args.len() != 1 {
                    return Err(EvalError::InvalidArgumentCount);
                }
                match &args[0] {
                    Value::Cons(car, _) => Ok((**car).clone()),
                    _ => Err(EvalError::InvalidArgumentType),
                }
            }

            BuiltinFunction::Cdr => {
                if args.len() != 1 {
                    return Err(EvalError::InvalidArgumentCount);
                }
                match &args[0] {
                    Value::Cons(_, cdr) => Ok((**cdr).clone()),
                    _ => Err(EvalError::InvalidArgumentType),
                }
            }

            BuiltinFunction::Cons => {
                if args.len() != 2 {
                    return Err(EvalError::InvalidArgumentCount);
                }
                Ok(Value::Cons(
                        Rc::new(args[0].clone()),
                        Rc::new(args[1].clone())
                ))
            }
        }
    }
}


// クロージャ
#[derive(Debug, Clone)]
pub struct Closure {
    params: Vec<String>,
    body: SExpr,
    env: Rc<RefCell<Environment>>,
}

// Rc比較のためのPartialEq実装
impl PartialEq for Closure {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}


// Environment
#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Self {
            values: HashMap::new(),
            parent: None,
        };

        // 組み込み関数の登録
        env.define("atom".to_string(), Value::Builtin(BuiltinFunction::Atom));
        env.define("eq".to_string(), Value::Builtin(BuiltinFunction::Eq));
        env.define("car".to_string(), Value::Builtin(BuiltinFunction::Car));
        env.define("cdr".to_string(), Value::Builtin(BuiltinFunction::Cdr));
        env.define("cons".to_string(), Value::Builtin(BuiltinFunction::Cons));

        env
    }

    pub fn with_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn lookup(&self, name: &str) -> Option<value> {
        if let Some(val) = self.values.get(name) {
            // まずは現環境から探す
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            // 現環境に名前がなくて，親の環境がある場合
            parent.borrow().lookup(name)
        } else {
            None
        }
    }
}


// evaluatorのエラー
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    UndefinedSymbol(String),
    InvalidArgumentCount,
    InvalidArgumentType,
    NotCallable,
    InvalidSpecialForm,
    InvalidLambda,
    // 必要に応じて追加する
}


// evaluator
pub fn eval(
    expr: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {
    match expr {
        SExpr::Nil => Ok(Value::Nil),
        SExpr::Atom(name) => {
            // Tは真値
            if name == "T" {
                return Ok(Value::Atom("T".to_string()));
            }
            // 環境からシンボルを探す
            env.borrow().lookup(name).ok_or_else(|| EvalError::UndefinedSymbol(name.clone()))
        }

        SExpr::Cons(operator, arguments) => {
            if let SExpr::Atom(op_name) = &**operator {
                match op_name.as_str() {
                    "quote" | "cond" | "lambda" | "define" => {
                        return eval_special_form(op_name, arguments, env);
                    }
                    _ => {}
                }
            }
            eval_application(operator, arguments, env)
        }
    }
}


// 特殊形式/Special Formの実装
fn eval_special_form(
    name: &str,
    args: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {
    match name {
        "quote" => eval_quote(args),
        "cond" => eval_cond(args, env),
        "lambda" => eval_lambda(args, env),
        "deifne" => eval_define(args, env),
        _ => Err(EvalError::InvalidSpecialForm),
    }
}
