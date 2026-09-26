/*
 * evaluator.rs
*/

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use crate::parser::SExpr;


// 評価する値の種類
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Atom(String),
    Nil,
    Cons(Rc<Value>, Rc<Value>),

    Builtin(BuiltinFunction),
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
                    (Value::Nil, Value::Nil) => Ok(Value::Atom("T".to_string())),
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

    pub fn lookup(&self, name: &str) -> Option<Value> {
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

fn eval_quote(args: &SExpr) -> Result<Value, EvalError> {
    let list = extract_list(args)?;
    if list.len() != 1 {
        return Err(EvalError::InvalidArgumentCount);
    }
    Ok(quote_to_value(list[0]))
}

fn eval_cond(clauses: &SExpr, env: Rc<RefCell<Environment>>) -> Result<Value, EvalError> {
    for clause in extract_list(clauses)? {
        let clause_list = extract_list(clause)?;
        if clause_list.is_empty() {
            continue;
        }

        let condition = clause_list[0];
        let cond_val = eval(condition, Rc::clone(&env))?;

        if cond_val != Value::Nil {
            if clause_list.len() > 1 {
                let mut res = Value::Nil;
                for expr in clause_list.iter().skip(1) {
                    res = eval(expr, Rc::clone(&env))?;
                }
                return Ok(res);
            } else {
                return Ok(cond_val);
            }
        }
    }
    Ok(Value::Nil)
}

fn eval_lambda(args: &SExpr, env: Rc<RefCell<Environment>>) -> Result<Value, EvalError> {
    let list = extract_list(args)?;
    if list.len() != 2 {
        return Err(EvalError::InvalidLambda);
    }

    let mut params = Vec::new();
    for p in extract_list(list[0])? {
        match p {
            SExpr::Atom(name) => params.push(name.clone()),
            _ => return Err(EvalError::InvalidLambda),
        }
    }

    Ok(Value::Lambda(Rc::new(Closure {
        params,
        body: list[1].clone(),
        env,
    })))
}

fn eval_define(args: &SExpr, env: Rc<RefCell<Environment>>) -> Result<Value, EvalError> {
    let list = extract_list(args)?;
    if list.len() != 2 {
        return Err(EvalError::InvalidArgumentCount);
    }

    let name = match list[0] {
        SExpr::Atom(n) => n.clone(),
        _ => return Err(EvalError::InvalidArgumentType),
    };

    let value = eval(list[1], Rc::clone(&env))?;
    env.borrow_mut().define(name, value.clone());
    Ok(value)
}


// function application
fn eval_application(
    operator: &SExpr,
    arguments: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Value, EvalError> {
    let function = eval(operator, Rc::clone(&env))?;
    let args_evaluated = eval_arguments(arguments, env)?;
    apply(function, args_evaluated)
}

fn eval_arguments(
    args: &SExpr,
    env: Rc<RefCell<Environment>>,
) -> Result<Vec<Value>, EvalError> {
    let list = extract_list(args)?;
    let mut evaluated = Vec::new();
    for expr in list {
        evaluated.push(eval(expr, Rc::clone(&env))?);
    }
    Ok(evaluated)
}

fn apply(function: Value, arguments: Vec<Value>) -> Result<Value, EvalError> {
    match function {
        Value::Builtin(builtin) => builtin.call(&arguments),
        Value::Lambda(closure) => apply_lambda(&closure, arguments),
        _ => Err(EvalError::NotCallable),
    }
}

fn apply_lambda(closure: &Closure, arguments: Vec<Value>) -> Result<Value, EvalError> {
    if closure.params.len() != arguments.len() {
        return Err(EvalError::InvalidArgumentCount);
    }

    let mut new_env = Environment::with_parent(Rc::clone(&closure.env));
    for (param, arg) in closure.params.iter().zip(arguments.into_iter()) {
        new_env.define(param.clone(), arg);
    }

    eval(&closure.body, Rc::new(RefCell::new(new_env)))
}


// ユーティティ
fn quote_to_value(expr: &SExpr) -> Value {
    match expr {
        SExpr::Nil => Value::Nil,
        SExpr::Atom(name) => Value::Atom(name.clone()),
        SExpr::Cons(car, cdr) => Value::Cons(
            Rc::new(quote_to_value(car)),
            Rc::new(quote_to_value(cdr)),
        ),
    }
}

// S式を平坦なリスト構造のイテレーション用に抽出する
fn extract_list(expr: &SExpr) -> Result<Vec<&SExpr>, EvalError> {
    let mut elements= Vec::new();
    let mut current = expr;
    loop {
        match current {
            SExpr::Nil => break,
            SExpr::Cons(car, cdr) => {
                elements.push(&**car);
                current = &**cdr;
            }
            SExpr::Atom(_) => return Err(EvalError::InvalidArgumentType),
        }
    }
    Ok(elements)
}
