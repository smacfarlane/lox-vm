use std::str::FromStr;

use crate::error::*;
use crate::token::{Token, TokenType};

use anyhow::Result;

#[derive(Debug)]
pub(crate) struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub(crate) fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub(crate) fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while self.peek().is_some() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "".to_string(), self.line));

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<()> {
        if let Some(c) = self.next() {
            match c {
                '(' => self.add_token(TokenType::LeftParen),
                ')' => self.add_token(TokenType::RightParen),
                '{' => self.add_token(TokenType::LeftBrace),
                '}' => self.add_token(TokenType::RightBrace),
                ',' => self.add_token(TokenType::Comma),
                '.' => self.add_token(TokenType::Dot),
                '-' => self.add_token(TokenType::Minus),
                '+' => self.add_token(TokenType::Plus),
                ';' => self.add_token(TokenType::Semicolon),
                '*' => self.add_token(TokenType::Star),
                '!' => {
                    if self.next_is('=') {
                        self.add_token(TokenType::BangEqual)
                    } else {
                        self.add_token(TokenType::Bang)
                    }
                }
                '=' => {
                    if self.next_is('=') {
                        self.add_token(TokenType::EqualEqual)
                    } else {
                        self.add_token(TokenType::Equal)
                    }
                }
                '<' => {
                    if self.next_is('=') {
                        self.add_token(TokenType::LessEqual)
                    } else {
                        self.add_token(TokenType::Less)
                    }
                }
                '>' => {
                    if self.next_is('=') {
                        self.add_token(TokenType::GreaterEqual)
                    } else {
                        self.add_token(TokenType::Greater)
                    }
                }
                '/' => {
                    if self.next_is('/') {
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            let _ = self.next();
                        }
                    } else {
                        self.add_token(TokenType::Slash)
                    }
                }
                ' ' | '\r' | '\t' => { /* ignore whitespace */ }
                '\n' => self.line += 1,
                '"' => self.add_string()?,
                n if n.is_ascii_digit() => self.add_number()?,
                i if (i.is_ascii_alphabetic() || i == '_') => self.add_identifier()?,
                _ => todo!(),
            }
        }

        Ok(())
    }

    fn add_token(&mut self, t: TokenType) {
        let lexeme = self
            .source
            .chars()
            .skip(self.start)
            .take(self.current - self.start)
            .collect::<String>();
        self.tokens.push(Token::new(t, lexeme, self.line));
    }

    fn add_string(&mut self) -> Result<()> {
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

        self.add_token(TokenType::String);
        Ok(())
    }

    fn add_number(&mut self) -> Result<()> {
        while self.peek().filter(char::is_ascii_digit).is_some() {
            let _ = self.next();
        }
        if self.peek() == Some('.') && self.peek_next().filter(char::is_ascii_digit).is_some() {
            let _ = self.next();

            while self.peek().filter(char::is_ascii_digit).is_some() {
                let _ = self.next();
            }
        }

        self.add_token(TokenType::Number);
        Ok(())
    }

    fn add_identifier(&mut self) -> Result<()> {
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
            self.add_token(token_type);
        } else {
            self.add_token(TokenType::Identifier);
        }

        Ok(())
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

    // TODO: Rename this
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
        let tokens = scanner.scan_tokens().unwrap();
        let mut iter = tokens.iter();

        assert_eq!(TokenType::Plus, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Minus, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Dot, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Comma, iter.next().unwrap().token_type);
        assert_eq!(TokenType::LeftParen, iter.next().unwrap().token_type);
        assert_eq!(TokenType::LeftBrace, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Semicolon, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Star, iter.next().unwrap().token_type);
        assert_eq!(TokenType::RightBrace, iter.next().unwrap().token_type);
        assert_eq!(TokenType::RightParen, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Greater, iter.next().unwrap().token_type);
        assert_eq!(TokenType::GreaterEqual, iter.next().unwrap().token_type);
        assert_eq!(TokenType::EqualEqual, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Bang, iter.next().unwrap().token_type);
        assert_eq!(TokenType::BangEqual, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Equal, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Less, iter.next().unwrap().token_type);
        assert_eq!(TokenType::LessEqual, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Slash, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Eof, iter.next().unwrap().token_type);
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_comments() {
        let input = String::from("// This should be ignored");

        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens().unwrap();
        let mut iter = tokens.iter();

        assert_eq!(TokenType::Eof, iter.next().unwrap().token_type);
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_whitespace() {
        let input = String::from(" \r\r\t\r  \t");

        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens().unwrap();
        let mut iter = tokens.iter();

        assert_eq!(TokenType::Eof, iter.next().unwrap().token_type);
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_newlines() {
        let input = String::from("\n\n\n");
        let mut scanner = Scanner::new(input);
        let _ = scanner.scan_tokens();

        assert_eq!(4, scanner.line)
    }

    #[test]
    fn test_string() {
        // TODO: Investigate why I thought this was correct
        // let input = String::from("\"abc\n123\"");
        let input = String::from("\"abc\n123\"");
        let expected = String::from("abc\n123");

        let mut scanner = Scanner::new(input.clone());
        let _ = scanner.scan_tokens();

        assert_eq!(
            TokenType::String,
            scanner.tokens.first().unwrap().token_type
        )
    }

    #[test]
    fn test_unterminated_string() {
        let input = String::from("\"abc");

        let mut scanner = Scanner::new(input.clone());
        let result = scanner.scan_tokens();
        assert!(result.is_err());
    }

    #[test]
    fn test_numbers() {
        let inputs = vec![123 as f64, 4567.2301];

        for input in inputs {
            let mut scanner = Scanner::new(input.to_string());
            let _ = scanner.scan_tokens();

            assert_eq!(
                TokenType::Number,
                scanner.tokens.first().unwrap().token_type
            )
        }
    }

    #[test]
    fn test_identifiers_and_keywords() {
        let input = r#"and
                       class
                       else
                       false
                       for
                       fun
                       if
                       nil
                       or
                       print
                       return
                       super
                       this
                       true
                       var
                       while
                       andy
                       while_true"#
            .to_string();

        let mut scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens().unwrap();
        let mut iter = tokens.iter();

        assert_eq!(TokenType::And, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Class, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Else, iter.next().unwrap().token_type);
        assert_eq!(TokenType::False, iter.next().unwrap().token_type);
        assert_eq!(TokenType::For, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Fun, iter.next().unwrap().token_type);
        assert_eq!(TokenType::If, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Nil, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Or, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Print, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Return, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Super, iter.next().unwrap().token_type);
        assert_eq!(TokenType::This, iter.next().unwrap().token_type);
        assert_eq!(TokenType::True, iter.next().unwrap().token_type);
        assert_eq!(TokenType::Var, iter.next().unwrap().token_type);
        assert_eq!(TokenType::While, iter.next().unwrap().token_type);
        let token = iter.next().unwrap();
        assert_eq!(TokenType::Identifier, token.token_type);
        assert_eq!("andy", token.lexeme);
        let token = iter.next().unwrap();
        assert_eq!(TokenType::Identifier, token.token_type);
        assert_eq!("while_true", token.lexeme);
    }
}
