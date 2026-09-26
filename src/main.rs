/*
 * main.rs
 */

use std::cell::RefCell;
use std::rc::Rc;

mod evaluator;
mod lexer;
mod parser;

use evaluator::{eval, Environment, Value};
use lexer::tokenize;
use parser::Parser;

fn main() {
    // 実行するLispコード（無名関数を定義して即座に呼び出す例）
    let code = "(define x ((lambda (x y) (cons x y)) '(A . B) '(C D)))";
    println!("--- Input Code ---");
    println!("{}", code);
    println!();

    // 1. 字句解析 (Lexing)
    let tokens = match tokenize(code) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("LexError: {:?}", e);
            return;
        }
    };

    // 2. 構文解析 (Parsing)
    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("ParseError: {:?}", e);
            return;
        }
    };

    // 3. 評価 (Evaluating)
    // グローバル環境を初期化（組み込み関数が登録される）
    let global_env = Rc::new(RefCell::new(Environment::new()));
    
    println!("--- Evaluation Result ---");
    match eval(&ast, global_env) {
        Ok(result) => {
            print_value(&result);
            println!();
        }
        Err(e) => {
            eprintln!("EvalError: {:?}", e);
        }
    }
}

// 評価結果(Value)をLispのドット対記法で標準出力するヘルパー関数
fn print_value(val: &Value) {
    match val {
        Value::Atom(s) => print!("{}", s),
        Value::Nil => print!("nil"),
        Value::Cons(car, cdr) => {
            print!("(");
            print_value(car);
            print!(" . ");
            print_value(cdr);
            print!(")");
        }
        Value::Builtin(_) => print!("<builtin>"),
        Value::Lambda(_) => print!("<lambda>"),
    }
}