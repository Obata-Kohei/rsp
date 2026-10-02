use std::{cell::RefCell, println};

use rsp::{evaluator::Environment, parser::SExpr};

pub enum ReplStatus {
	Handled,
	Exit,
}

pub fn handle_repl_command(expr: &SExpr, env: &RefCell<Environment>) -> Option<ReplStatus> {
	let SExpr::Cons(car, cdr) = expr else {return None;};
	let SExpr::Atom(name) = &**car else {return None;};

	// 全てのREPLコマンドに引数がない時点ではこのコードは有効である
	// helpコマンドに引数を持たせるなど，引数のあるコマンドが出てきたらこれを消す
	if **cdr != SExpr::Nil {
		return None;
	}

	match name.as_str() {
		"exit" => Some(eval_exit()),
		"help" => Some(eval_help()),
		"reset" => Some(eval_reset(env)),
		"env" => Some(eval_env(env)),
		_ => None,
	}
}

fn eval_exit() -> ReplStatus {
	ReplStatus::Exit
}

fn eval_help() -> ReplStatus {
	println!("REPL Commands:");
	println!("  (exit)       : Exit the REPL");
	println!("  (help)       : Show this help message");
	println!("  (reset-env)  : Reset global environment to default");
	ReplStatus::Handled
}

// 環境のリセット
fn eval_reset(env: &RefCell<Environment>) -> ReplStatus {
	*env.borrow_mut() = Environment::new();
	println!("Environment reset.");
	ReplStatus::Handled
}

// 環境の一覧表示
fn eval_env(env: &RefCell<Environment>) -> ReplStatus {
	println!("Current Environment:");
	println!("{:#?}", *env.borrow());
	ReplStatus::Handled
}
