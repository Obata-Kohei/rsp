/*
 * parser.rs
*/

use crate::lexer::Token;

const NIL: &str = "nil";
const QUOTE: &str = "quote";

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
	Atom(String),
	Nil,
	Cons(Box<SExpr>, Box<SExpr>),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
	UnexpectedToken(Token),
	UnexpectedEof,
	UnexpectedCloseParen,
	InvalidDottedPair,
}

pub struct Parser {
	tokens: Vec<Token>,
	position: usize,
}

impl Parser {
	pub fn new(tokens: Vec<Token>) -> Self {
		Self { tokens, position: 0 }
	}

	pub fn parse(&mut self) -> Result<SExpr, ParseError> {
		let expr = self.parse_expr()?;

		if self.position < self.tokens.len() {
			return Err(ParseError::UnexpectedToken(
				self.tokens[self.position].clone(),
			));
		}

		Ok(expr)
	}

	fn parse_expr(&mut self) -> Result<SExpr, ParseError> {
		let token = self.next_token()?;

		match token {
			Token::Symbol(s) if s == NIL => Ok(SExpr::Nil),
			Token::Symbol(symbol) => Ok(SExpr::Atom(symbol)),

			// 構文糖衣: 'expr -> (quote expr)
			Token::Quote => {
				let expr = self.parse_expr()?;
				Ok(SExpr::Cons(
					Box::new(SExpr::Atom(QUOTE.to_string())),
					Box::new(SExpr::Cons(Box::new(expr), Box::new(SExpr::Nil)))
				))
			}

			Token::LParen => self.parse_list(),

			// ')' や '.' はパースエラー
			Token::RParen => Err(ParseError::UnexpectedCloseParen),
			Token::Dot => Err(ParseError::InvalidDottedPair),
		}
	}

	// '(' の後の要素をパースする
	fn parse_list(&mut self) -> Result<SExpr, ParseError> {
		// ()はnilに
		if self.peek_token() == Some(&Token::RParen) {
			self.next_token()?;
			return Ok(SExpr::Nil);
		}

		// 最初の要素をパースする
		let head = self.parse_expr()?;

		// 次のトークンを確認してCons cell(ドット対)やリスト(構文糖衣)をパースする
		match self.peek_token() {
			// Cons cell (A . B)
			Some(Token::Dot) => {
				self.next_token()?;  // '.'を消費
				let tail = self.parse_expr()?;

				// Cons cellは必ず')'で終わる
				if self.next_token()? != Token::RParen {
					return Err(ParseError::InvalidDottedPair);
				}

				Ok(SExpr::Cons(Box::new(head), Box::new(tail)))
			}

			// 構文糖衣 (A), (A B C) や None の処理
			_ =>  {
				let tail = self.parse_list()?;
				Ok(SExpr::Cons(Box::new(head), Box::new(tail)))
			}
		}
	}

	fn next_token(&mut self) -> Result<Token, ParseError> {
		if self.position >= self.tokens.len() {
			return Err(ParseError::UnexpectedEof);
		}
		let token = self.tokens[self.position].clone();
		self.position += 1;
		Ok(token)
	}

	fn peek_token(&self) -> Option<&Token> {
		self.tokens.get(self.position)
	}
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse(input: &str) -> SExpr {
        let tokens = tokenize(input).unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap()
    }

    #[test]
    fn parse_atom() {
        assert_eq!(parse("A"), SExpr::Atom("A".to_string()));
    }

    #[test]
    fn parse_nil() {
        assert_eq!(parse("nil"), SExpr::Nil);
        assert_eq!(parse("()"), SExpr::Nil);
    }

    #[test]
    fn parse_dotted_pair() {
        assert_eq!(
            parse("(A . B)"),
            SExpr::Cons(
                Box::new(SExpr::Atom("A".to_string())),
                Box::new(SExpr::Atom("B".to_string()))
            )
        );
    }

    #[test]
    fn parse_single_element_list() {
        // (A) -> (A . nil)
        assert_eq!(
            parse("(A)"),
            SExpr::Cons(
                Box::new(SExpr::Atom("A".to_string())),
                Box::new(SExpr::Nil)
            )
        );
    }

    #[test]
    fn parse_list_sugar() {
        // (A B C) -> (A . (B . (C . nil)))
        assert_eq!(
            parse("(A B C)"),
            SExpr::Cons(
                Box::new(SExpr::Atom("A".to_string())),
                Box::new(SExpr::Cons(
                    Box::new(SExpr::Atom("B".to_string())),
                    Box::new(SExpr::Cons(
                        Box::new(SExpr::Atom("C".to_string())),
                        Box::new(SExpr::Nil)
                    ))
                ))
            )
        );
    }

    #[test]
    fn parse_quote_sugar() {
        // 'A -> (quote A) -> (quote . (A . nil))
        assert_eq!(
            parse("'A"),
            SExpr::Cons(
                Box::new(SExpr::Atom("quote".to_string())),
                Box::new(SExpr::Cons(
                    Box::new(SExpr::Atom("A".to_string())),
                    Box::new(SExpr::Nil)
                ))
            )
        );
    }

	#[test]
	fn parse_unexpected_close_paren() {
		let tokens = tokenize(")").unwrap();
		let mut parser = Parser::new(tokens);

		assert_eq!(
			parser.parse(),
			Err(ParseError::UnexpectedCloseParen)
		);
	}

	#[test]
	fn parse_unexpected_dot() {
		let tokens = tokenize(".").unwrap();
		let mut parser = Parser::new(tokens);

		assert_eq!(
			parser.parse(),
			Err(ParseError::InvalidDottedPair)
		);
	}

	#[test]
	fn parse_unclosed_list() {
		let tokens = tokenize("(A B").unwrap();
		let mut parser = Parser::new(tokens);

		assert_eq!(
			parser.parse(),
			Err(ParseError::UnexpectedEof)
		);
	}

	#[test]
	fn parse_invalid_dotted_pair() {
		let tokens = tokenize("(A . B C)").unwrap();
		let mut parser = Parser::new(tokens);

		assert_eq!(
			parser.parse(),
			Err(ParseError::InvalidDottedPair)
		);
	}

	#[test]
	fn parse_empty_list() {
		assert_eq!(
			parse("()"),
			SExpr::Nil
		);
	}

	#[test]
	fn parse_nested_list() {
		assert_eq!(
			parse("(A (B C))"),
			SExpr::Cons(
				Box::new(SExpr::Atom("A".to_string())),
				Box::new(
					SExpr::Cons(
						Box::new(
							SExpr::Cons(
								Box::new(SExpr::Atom("B".to_string())),
								Box::new(
									SExpr::Cons(
										Box::new(SExpr::Atom("C".to_string())),
										Box::new(SExpr::Nil),
									)
								),
							)
						),
						Box::new(SExpr::Nil),
					)
				),
			)
		);
	}

	#[test]
	fn parse_nested_quote() {
		assert_eq!(
			parse("'(A B)"),
			SExpr::Cons(
				Box::new(SExpr::Atom("quote".to_string())),
				Box::new(
					SExpr::Cons(
						Box::new(
							SExpr::Cons(
								Box::new(SExpr::Atom("A".to_string())),
								Box::new(
									SExpr::Cons(
										Box::new(SExpr::Atom("B".to_string())),
										Box::new(SExpr::Nil),
									)
								),
							)
						),
						Box::new(SExpr::Nil),
					)
				),
			)
		);
	}
}