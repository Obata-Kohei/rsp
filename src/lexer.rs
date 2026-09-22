/*
 * lexer.rs
*/

 #[derive(Debug, Clone, PartialEq)]
pub enum Token {
	LParen,  // (
	RParen,  // )
	Dot,  // .
	// Quote,  // '
	Symbol(String),  // A or nil or car etc...
}

#[derive(Debug)]
enum LexError {
	InvalidTokenError,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
	let mut token_list = Vec::new();
	let mut chars = input.chars().peekable();

	while let Some(&ch) = chars.peek() {
		match ch {
			// 空白文字は読み飛ばす
			c if c.is_whitespace() => {
				chars.next();
			}
			'(' => {
				token_list.push(Token::LParen);
				chars.next();
			}
			')' => {
				token_list.push(Token::RParen);
				chars.next();
			}
			'.' => {
				token_list.push(Token::Dot);
				chars.next();
			}
			/*
			'\'' => {
				token_list.push(Token::Quote);
				chars.next();
			}
			 */
			// 空白や区切り文字以外が続く限りまとめてSymbolとして扱う
			_ => {
				let mut symbol = String::new();
				while let Some(&c) = chars.peek() {
					if c.is_whitespace() || c == '(' || c == ')' || c == '.' /*|| c == '\''*/ {
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


#[test]
fn lex_test() {
	let code = "(car A)";
	assert_eq!(
		tokenize(code).unwrap(),
		vec![Token::LParen, Token::Symbol("car".to_string()), Token::Symbol("A".to_string()), Token::RParen]
	);
}

#[test]
fn lex_test2() {
	let code = "(car ((A . B) . (C . D)))";
	assert_eq!(
		tokenize(code).unwrap(),
		vec![Token::LParen, Token::Symbol("car".to_string()), 
				Token::LParen,
				Token::LParen, Token::Symbol("A".to_string()), Token::Dot, Token::Symbol("B".to_string()), Token::RParen,
				Token::Dot,
				Token::LParen, Token::Symbol("C".to_string()), Token::Dot, Token::Symbol("D".to_string()), Token::RParen,
				Token::RParen,
			Token::RParen,
			]
	)
}