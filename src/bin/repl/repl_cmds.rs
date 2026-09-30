use std::{cell::RefCell, rc::Rc, todo};

use rsp::{evaluator::Environment, parser::SExpr};

pub enum ReplAction {
	Continue,
	Exit,
}

pub fn handle_repl_command(expr: &SExpr, env: Rc<RefCell<Environment>>) -> Option<ReplAction> {
	let SExpr::Cons(car, cdr) = expr else {return None;};
	let SExpr::Atom(name) = &**car else {return None;};

	match name.as_str() {
		"exit" if **cdr == SExpr::Nil => eval_exit(),
		"help" if **cdr == SExpr::Nil => eval_help(),
		"reset" if **cdr == SExpr::Nil => eval_reset(env),
		"env" if **cdr == SExpr::Nil => eval_env(env),
		_ => None
	}
}

fn eval_exit() -> Option<ReplAction> {
	Some(ReplAction::Exit)
}

fn eval_help() -> Option<ReplAction> {
	println!("REPL Commands:");
	println!("  (exit)       : Exit the REPL");
	println!("  (help)       : Show this help message");
	println!("  (reset-env)  : Reset global environment to default");
	Some(ReplAction::Continue)
}

// 環境のリセット
fn eval_reset(env: Rc<RefCell<Environment>>) -> Option<ReplAction> {
	*env.borrow_mut() = Environment::new();
	println!("Environment reset.");
	Some(ReplAction::Continue)
}

// 環境の一覧表示
fn eval_env(env: Rc<RefCell<Environment>>) -> Option<ReplAction> {
	todo!()
}