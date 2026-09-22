/*
 * parser.rs
*/

use crate::lexer::Token;

#[derive(Debug)]
pub enum Data {
	Atom(String),
	Nil,
	Cons(Box<Data>, Box<Data>),
}

#[derive(Debug)]
pub enum ParseError {
	UnexpectedToken(String),
}

#[derive(Debug, Clone)]
pub struct Parser {
	tokens: Vec<Token>,
	position: usize,
}