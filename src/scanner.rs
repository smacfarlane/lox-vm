use std::str::FromStr;

use crate::error::*;
use crate::token::{Token, TokenType};

use anyhow::Result;

#[derive(Debug)]
pub(crate) struct Scanner {
    source: String,
    pub start: usize,
    pub current: usize,
    pub line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Result<Token> {
        self.skip_whitespace();
        self.start = self.current;
        if let Some(c) = self.next() {
            let token = match c {
                '(' => self.make_token(TokenType::LeftParen),
                ')' => self.make_token(TokenType::RightParen),
                '{' => self.make_token(TokenType::LeftBrace),
                '}' => self.make_token(TokenType::RightBrace),
                ',' => self.make_token(TokenType::Comma),
                '.' => self.make_token(TokenType::Dot),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                ';' => self.make_token(TokenType::Semicolon),
                '*' => self.make_token(TokenType::Star),
                '/' => self.make_token(TokenType::Slash),
                '!' => {
                    if self.next_is('=') {
                        self.make_token(TokenType::BangEqual)
                    } else {
                        self.make_token(TokenType::Bang)
                    }
                }
                '=' => {
                    if self.next_is('=') {
                        self.make_token(TokenType::EqualEqual)
                    } else {
                        self.make_token(TokenType::Equal)
                    }
                }
                '<' => {
                    if self.next_is('=') {
                        self.make_token(TokenType::LessEqual)
                    } else {
                        self.make_token(TokenType::Less)
                    }
                }
                '>' => {
                    if self.next_is('=') {
                        self.make_token(TokenType::GreaterEqual)
                    } else {
                        self.make_token(TokenType::Greater)
                    }
                }
                '"' => self.string()?,
                n if n.is_ascii_digit() => self.number()?,
                i if (i.is_ascii_alphabetic() || i == '_') => self.identifier()?,
                t => todo!("{}", t),
            };

            Ok(token)
        } else {
            Ok(self.make_token(TokenType::Eof))
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            if let Some(c) = self.peek() {
                match c {
                    ' ' | '\r' | '\t' => {
                        self.next();
                    }
                    '\n' => {
                        self.line += 1;
                        self.next();
                    }
                    '/' => {
                        if self.peek_next() == Some('/') {
                            while self.peek().is_some() && self.peek().unwrap() != '\n' {
                                self.next();
                            }
                        } else {
                            return;
                        }
                    }
                    _ => return,
                }
            } else {
                return;
            }
        }
    }

    fn make_token(&mut self, t: TokenType) -> Token {
        let lexeme = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect::<String>();
        Token::new(t, lexeme, self.line)
    }

    fn string(&mut self) -> Result<Token> {
        while let Some(c) = self.peek().filter(|c| *c != '"') {
            if c == '\n' {
                self.line += 1;
            }
            let _ = self.next();
        }

        if self.peek().is_none() {
            return Err(ParseError::UnterminatedString(ErrorLoc {
                line: self.line,
                at: self.start,
            })
            .into());
        }
        let _ = self.next();

        Ok(self.make_token(TokenType::String))
    }

    fn number(&mut self) -> Result<Token> {
        while self.peek().filter(char::is_ascii_digit).is_some() {
            let _ = self.next();
        }
        if self.peek() == Some('.') && self.peek_next().filter(char::is_ascii_digit).is_some() {
            let _ = self.next();

            while self.peek().filter(char::is_ascii_digit).is_some() {
                let _ = self.next();
            }
        }

        Ok(self.make_token(TokenType::Number))
    }

    fn identifier(&mut self) -> Result<Token> {
        while self
            .peek()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || c.is_ascii_digit())
            .is_some()
        {
            let _ = self.next();
        }

        let text = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect::<String>();

        if let Ok(token_type) = TokenType::from_str(&text) {
            Ok(self.make_token(token_type))
        } else {
            Ok(self.make_token(TokenType::Identifier))
        }
    }

    fn next(&mut self) -> Option<char> {
        self.current += 1;
        self.source.chars().nth(self.current - 1)
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current)
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }

    fn next_is(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.current += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scanner() {
        let input = String::from("+-.,({;*})>>===!!==<<=/");

        let mut scanner = Scanner::new(input);

        assert_eq!(TokenType::Plus, scanner.scan_token().unwrap().token_type);
        assert_eq!(TokenType::Minus, scanner.scan_token().unwrap().token_type);
        assert_eq!(TokenType::Dot, scanner.scan_token().unwrap().token_type);
        assert_eq!(TokenType::Comma, scanner.scan_token().unwrap().token_type);
        assert_eq!(
            TokenType::LeftParen,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(
            TokenType::LeftBrace,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(
            TokenType::Semicolon,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(TokenType::Star, scanner.scan_token().unwrap().token_type);
        assert_eq!(
            TokenType::RightBrace,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(
            TokenType::RightParen,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(TokenType::Greater, scanner.scan_token().unwrap().token_type);
        assert_eq!(
            TokenType::GreaterEqual,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(
            TokenType::EqualEqual,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(TokenType::Bang, scanner.scan_token().unwrap().token_type);
        assert_eq!(
            TokenType::BangEqual,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(TokenType::Equal, scanner.scan_token().unwrap().token_type);
        assert_eq!(TokenType::Less, scanner.scan_token().unwrap().token_type);
        assert_eq!(
            TokenType::LessEqual,
            scanner.scan_token().unwrap().token_type
        );
        assert_eq!(TokenType::Slash, scanner.scan_token().unwrap().token_type);
        assert_eq!(TokenType::Eof, scanner.scan_token().unwrap().token_type);
    }

    // #[test]
    // fn test_comments() {
    //     let input = String::from("// This should be ignored");
    //
    //     let mut scanner = Scanner::new(input);
    //     let tokens = scanner.scan_tokens().unwrap();
    //     let mut iter = tokens.iter();
    //
    //     assert_eq!(TokenType::Eof, iter.next().unwrap().token_type);
    //     assert_eq!(None, iter.next());
    // }
    //
    // #[test]
    // fn test_whitespace() {
    //     let input = String::from(" \r\r\t\r  \t");
    //
    //     let mut scanner = Scanner::new(input);
    //     let tokens = scanner.scan_tokens().unwrap();
    //     let mut iter = tokens.iter();
    //
    //     assert_eq!(TokenType::Eof, iter.next().unwrap().token_type);
    //     assert_eq!(None, iter.next());
    // }
    //
    // #[test]
    // fn test_newlines() {
    //     let input = String::from("\n\n\n");
    //     let mut scanner = Scanner::new(input);
    //     let _ = scanner.scan_tokens();
    //
    //     assert_eq!(4, scanner.line)
    // }
    //
    // #[test]
    // fn test_string() {
    //     // TODO: Investigate why I thought this was correct
    //     // let input = String::from("\"abc\n123\"");
    //     let input = String::from("\"abc\n123\"");
    //     let expected = String::from("abc\n123");
    //
    //     let mut scanner = Scanner::new(input.clone());
    //     let _ = scanner.scan_tokens();
    //
    //     assert_eq!(
    //         TokenType::String,
    //         scanner.tokens.first().unwrap().token_type
    //     )
    // }
    //
    // #[test]
    // fn test_unterminated_string() {
    //     let input = String::from("\"abc");
    //
    //     let mut scanner = Scanner::new(input.clone());
    //     let result = scanner.scan_tokens();
    //     assert!(result.is_err());
    // }
    //
    // #[test]
    // fn test_numbers() {
    //     let inputs = vec![123 as f64, 4567.2301];
    //
    //     for input in inputs {
    //         let mut scanner = Scanner::new(input.to_string());
    //         let _ = scanner.scan_tokens();
    //
    //         assert_eq!(
    //             TokenType::Number,
    //             scanner.tokens.first().unwrap().token_type
    //         )
    //     }
    // }
    //
    // #[test]
    // fn test_identifiers_and_keywords() {
    //     let input = r#"and
    //                    class
    //                    else
    //                    false
    //                    for
    //                    fun
    //                    if
    //                    nil
    //                    or
    //                    print
    //                    return
    //                    super
    //                    this
    //                    true
    //                    var
    //                    while
    //                    andy
    //                    while_true"#
    //         .to_string();
    //
    //     let mut scanner = Scanner::new(input);
    //     let tokens = scanner.scan_tokens().unwrap();
    //     let mut iter = tokens.iter();
    //
    //     assert_eq!(TokenType::And, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Class, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Else, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::False, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::For, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Fun, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::If, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Nil, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Or, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Print, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Return, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Super, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::This, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::True, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::Var, iter.next().unwrap().token_type);
    //     assert_eq!(TokenType::While, iter.next().unwrap().token_type);
    //     let token = iter.next().unwrap();
    //     assert_eq!(TokenType::Identifier, token.token_type);
    //     assert_eq!("andy", token.lexeme);
    //     let token = iter.next().unwrap();
    //     assert_eq!(TokenType::Identifier, token.token_type);
    //     assert_eq!("while_true", token.lexeme);
    // }
}
