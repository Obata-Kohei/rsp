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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_atom() {
        assert_eq!(
            tokenize("ABC").unwrap(),
            vec![Token::Symbol("ABC".to_string())]
        );
    }

    #[test]
    fn tokenize_multiple_symbols() {
        assert_eq!(
            tokenize("A B C").unwrap(),
            vec![
                Token::Symbol("A".to_string()),
                Token::Symbol("B".to_string()),
                Token::Symbol("C".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_parentheses() {
        assert_eq!(
            tokenize("(A B)").unwrap(),
            vec![
                Token::LParen,
                Token::Symbol("A".to_string()),
                Token::Symbol("B".to_string()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn tokenize_dotted_pair() {
        assert_eq!(
            tokenize("(A . B)").unwrap(),
            vec![
                Token::LParen,
                Token::Symbol("A".to_string()),
                Token::Dot,
                Token::Symbol("B".to_string()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn tokenize_quote() {
        assert_eq!(
            tokenize("'A").unwrap(),
            vec![
                Token::Quote,
                Token::Symbol("A".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_nested_expression() {
        assert_eq!(
            tokenize("((lambda (x) (atom x)) 'Hello)").unwrap(),
            vec![
                Token::LParen,
                Token::LParen,
                Token::Symbol("lambda".to_string()),
                Token::LParen,
                Token::Symbol("x".to_string()),
                Token::RParen,
                Token::LParen,
                Token::Symbol("atom".to_string()),
                Token::Symbol("x".to_string()),
                Token::RParen,
                Token::RParen,
                Token::Quote,
                Token::Symbol("Hello".to_string()),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn tokenize_whitespace() {
        assert_eq!(
            tokenize(" \t\n  A \n B ").unwrap(),
            vec![
                Token::Symbol("A".to_string()),
                Token::Symbol("B".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_empty_input() {
        assert_eq!(
            tokenize("").unwrap(),
            Vec::<Token>::new()
        );
    }
}