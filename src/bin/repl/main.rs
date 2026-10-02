use std::io::{self, Write};
use std::cell::RefCell;
use std::{eprintln, print, println};
use std::rc::Rc;

use rsp::lexer::tokenize;
use rsp::parser::Parser;
use rsp::evaluator::{eval, Environment, Value};

use crate::repl_cmds::{ReplStatus, handle_repl_command};

mod repl_cmds;

fn main() {
	// REPL全体で共有する環境
	let global_env = Rc::new(RefCell::new(Environment::new()));

	println!("< Lisp REPL >");
	println!("Type Ctrl-D to exit.");

	'repl: loop {
		io::stdout().flush().expect("stdout should be flushed...");

		// ひとつのS式を完成するまで入力する
		let input = match read_expression() {
			Some(input) => input,
			None => break,
		};

		// 空行を無視する
		if input.trim().is_empty() {
			continue;
		}

		// 字句解析
		let tokens = match tokenize(&input) {
			Ok(tokens) => tokens,
			Err(e) => {
				eprintln!("LexError: {:?}", e);
				continue;
			}
		};

		// 構文解析
		let mut parser = Parser::new(tokens);
		let expressions = match parser.parse() {
			Ok(expressions) => expressions,
			Err(e) => {
				eprintln!("ParseError: {:?}", e);
				continue;
			}
		};

		// S式を順番に評価
		for expr in expressions {
			// まずREPL特有のコマンドか検査して処理する
			match handle_repl_command(&expr, &global_env) {
				Some(ReplStatus::Exit) => break 'repl,  // REPL全体を終了
				Some(ReplStatus::Handled) => continue,  // REPLコマンドを実行したので以降のevalはスキップ
				None => {}  // REPLコマンドではない場合は通常のevalへ
			}

			// eval
			match eval(&expr, Rc::clone(&global_env)) {
				Ok(value) => {
					print_value(&value);
					println!();
				}
				Err(e) => {
					eprintln!("EvalError: {:?}", e);
				}
			}
		}
	}

	println!("bye bye");
}

// 1つ以上のS式が完成するまで標準入力から読み込む．
// 括弧の対応を見て
//
// > (cond
// ... ((eq 'A 'B) 'first)
// ... ((eq 'A 'A) 'second)
// ... (T 'default))
//
// のような複数行入力を可能にする．
// EOF (Ctrl-D) が入力された場合は None を返す。
fn read_expression() -> Option<String> {
	let mut input = String::new();
	let mut paren_depth = 0_usize;
	let mut has_input = false;

	loop {
		// 最初の行は通常のプロンプト，
		// 継続入力では別のプロンプトを表示
		if paren_depth == 0 {
			print!("> ");
		} else {
			print!("... ");
		}

		io::stdout().flush().expect("stdout should be flushed...");

		let mut line = String::new();
		match io::stdin().read_line(&mut line) {
			Ok(0) => {
				// EOF
				if !has_input {
					println!();
					return None;
				}

				// 入力途中でCtrl-Dされた場合
				// 閉じていないS式を評価せずに入力を破棄する
				if paren_depth > 0 {
					eprintln!();
					eprintln!("Unexpected EOF: unclosed parenthesis");
					return None;
				}

				return Some(input);
			}

			Ok(_) => {
				has_input = true;

				input.push_str(&line);
				for ch in line.chars() {
					match ch {
						'(' => {
							paren_depth += 1;
						}

						')' => {
							if paren_depth == 0 {
								// 閉じかっこが多すぎる
								eprintln!("ParseError: unexpected ')'");
								return Some(input);
							}

							paren_depth -= 1;
						}

						_ => {}
					}
				}

				// かっこがすべて閉じたら入力完了
				if paren_depth == 0 {
					return Some(input);
				}
			}

			Err(e) => {
				eprintln!("InputError: {}", e);
				return None;
			}
		}
	}
}

fn print_value(value: &Value) {
	match value {
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