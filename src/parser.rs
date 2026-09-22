/*
 * parser.rs
*/

use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
	Atom(String),
	Nil,
	Cons(Box<SExpr>, Box<SExpr>),
}

#[derive(Debug)]
pub enum ParseError {
	UnexpectedToken(Token),
	UnexpectedEof,
	UnexpectedCloseParen,
	InvalidDottedPair,
}

#[derive(Debug, Clone)]
pub struct Parser {
	tokens: Vec<Token>,
	position: usize,
}

impl Parser {
	pub fn new(tokens: Vec<Token>) -> Self {
		Self {tokens, position: 0}
	}

	pub fn parse(&mut self) -> Result<SExpr, ParseError> {
		let expr = self.parse_expr()?;

		// parse_expr() の後にTokenが残っていたらエラー
		if self.position < self.tokens.len() {
			return Err(ParseError::UnexpectedToken(
                self.tokens[self.position].clone()
            ));
		}

		Ok(expr)
	}

	// 1つのS式を読み込む
	fn parse_expr(&mut self) -> Result<SExpr, ParseError> {
		let token = self.next_token()?;

		match token {
			// nilのパターン
			Token::Symbol(s) if s == "nil" => {
				Ok(SExpr::Nil)
			}

			// Symbol -> Atom
			Token::Symbol(symbol) => {
				Ok(SExpr::Atom(symbol))
			}

			// `(`が来たらリストとしてパースする
			Token::LParen => {
				self.parse_list()
			}

			// `)`閉じかっこはエラー
			Token::RParen => {
				Err(ParseError::UnexpectedCloseParen)
			}

			// `.`はエラー
			Token::Dot => {
				Err(ParseError::InvalidDottedPair)
			}
		}
	}

	// リストのパース
	fn parse_list(&mut self) -> Result<SExpr, ParseError> {
		// 空Cons cell `()` の場合は `nil` として扱う`
		if self.peek_token() == Some(&Token::RParen) {
			self.next_token();
			return Ok(SExpr::Nil);
		}

		// 最初の要素をパースする
		let first = self.parse_expr()?;
		// リストの残りを読み込む
		self.parse_list_tail(first)
	}

	fn parse_list_tail(&mut self, first: SExpr) -> Result<SExpr, ParseError> {
		match self.peek_token() {
			// (A . B)
			Some(Token::Dot) => {
				self.next_token();
				let second = self.parse_expr()?;

				match self.peek_token() {
					Some(Token::RParen) => {
						self.next_token();

						Ok(SExpr::Cons(
							Box::new(first),
							Box::new(second),
						))
					}

					_ => {
						Err(ParseError::InvalidDottedPair)
					}
				}
			}

			// (A)
			Some(Token::RParen) => {
				self.next_token();

				Ok(SExpr::Cons(
					Box::new(first),
				Box::new(SExpr::Nil)
				))
			}

			// (A B ...)
			Some(_) => {
				let second = self.parse_expr()?;
				let rest = self.parse_list_tail(second)?;

				Ok(SExpr::Cons(
					Box::new(first),
					Box::new(rest),
				))
			}

			// `)`がない
			None => {
				Err(ParseError::UnexpectedEof)
			}
		}
	}

	// 次のtokenを取得して，self.positionを進める
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
        assert_eq!(
            parse("A"),
            SExpr::Atom("A".to_string())
        );
    }

    #[test]
    fn parse_nil() {
        assert_eq!(
            parse("nil"),
            SExpr::Nil
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
    fn parse_cons() {
        assert_eq!(
            parse("(A . B)"),
            SExpr::Cons(
                Box::new(SExpr::Atom("A".to_string())),
                Box::new(SExpr::Atom("B".to_string())),
            )
        );
    }

    #[test]
    fn parse_list() {
        assert_eq!(
            parse("(A B C)"),
            SExpr::Cons(
                Box::new(SExpr::Atom("A".to_string())),
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
            )
        );
    }

    #[test]
    fn parse_nested_cons() {
        assert_eq!(
            parse("(A . (B . C))"),
            SExpr::Cons(
                Box::new(SExpr::Atom("A".to_string())),
                Box::new(
                    SExpr::Cons(
                        Box::new(SExpr::Atom("B".to_string())),
                        Box::new(SExpr::Atom("C".to_string())),
                    )
                ),
            )
        );
    }
}
