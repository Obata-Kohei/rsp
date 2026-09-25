/*
 * lexer.rs
*/

const LPAREN: char = '(';
const RPAREN: char = ')';
const DOT: char = '.';
const QUOTE: char = '\'';

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
	LParen,  // (
	RParen,  // )
	Dot,  // .
	Symbol(String),  // A, nil, car, etc ...
	Quote,  // ' 構文糖衣 '(A B C)
}

#[derive(Debug, PartialEq)]
pub enum LexError {
	InvalidTokenError,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
	let mut token_list = Vec::new();
	let mut chars = input.chars().peekable();

	while let Some(&ch) = chars.peek() {
		match ch {
			c if c.is_whitespace() => {
				chars.next();
			}
			LPAREN => {
				token_list.push(Token::LParen);
				chars.next();
			}
			RPAREN => {
				token_list.push(Token::RParen);
				chars.next();
			}
			DOT => {
				token_list.push(Token::Dot);
				chars.next();
			}
			QUOTE => {
				token_list.push(Token::Quote);
				chars.next();
			}
			_ => {
				let mut symbol = String::new();
				while let Some(&c) = chars.peek() {
					// 空白やトークンが出るまで続ける
					if c.is_whitespace() || c == LPAREN || c == RPAREN || c == DOT || c == QUOTE {
						break;
					}
					symbol.push(c);
					chars.next();
				}
				token_list.push(Token::Symbol(symbol));
			}
		}
	}

	Ok(token_list)
}