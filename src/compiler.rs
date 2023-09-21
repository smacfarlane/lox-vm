use crate::chunk::Value;
use crate::parse::{self, ParseFn, ParseRule, Parser, Precedence};
use crate::token::{Token, TokenType};
use crate::{Chunk, OpCode};

use anyhow::{anyhow, Result};

struct Compiler {
    parser: Parser,
    scanner: crate::scanner::Scanner,
    compiling_chunk: Chunk,
}

impl Compiler {
    fn new(source: String) -> Compiler {
        let scanner = crate::scanner::Scanner::new(source);
        Compiler {
            parser: Parser::new(),
            scanner,
            compiling_chunk: Chunk::new(),
        }
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.parser.previous.clone().unwrap(), message);
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(&self.parser.current.clone().unwrap(), message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
        if self.parser.panic_mode {
            return;
        }
        self.parser.panic_mode = true;
        let suffix = match token.token_type {
            TokenType::Eof => String::from("end"),
            _ => format!("'{}'", token.lexeme),
        };
        eprintln!("[line {}] Error at {}: {}", token.line, suffix, message);
        self.parser.had_error = true;
    }

    fn advance(&mut self) -> Result<()> {
        self.parser.advance();

        loop {
            match self.scan_token() {
                Ok(token) => {
                    self.parser.current = Some(token);
                    break;
                }
                Err(_) => self.error_at_current(&self.parser.current.clone().unwrap().lexeme),
            }
        }
        Ok(())
    }

    fn consume(&mut self, tt: TokenType, message: &str) -> Result<()> {
        match &self.parser.current {
            Some(token) => {
                if token.token_type == tt {
                    self.advance()
                } else {
                    self.error_at_current(message);
                    Err(anyhow!(message.to_owned()))
                }
            }
            None => Err(anyhow!("no current token")),
        }
    }

    fn scan_token(&mut self) -> Result<Token> {
        self.scanner.scan_token()
    }

    fn number(&mut self) {
        let value = self
            .parser
            .previous
            .clone()
            .expect("expected previous chunk")
            .lexeme;
        let value: Value = value
            .parse()
            .expect(&format!("unable to convert token to float {}", value));
        let _ = self.emit_constant(value);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&mut self) {
        self.expression();
        let _ = self.consume(TokenType::RightParen, "expected ')' after expression)");
    }

    fn unary(&mut self) {
        let operator_type = self
            .parser
            .previous
            .clone()
            .expect("expected previous token")
            .token_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let operator_type = self
            .parser
            .previous
            .clone()
            .expect("expected previous token")
            .token_type;
        let rule = self.get_rule(&operator_type);

        self.parse_precedence(rule.precedence.next()); // TODO: Offset by one (?)

        match operator_type {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn emit_byte<T>(&mut self, byte: T)
    where
        T: Into<u8> + std::fmt::Debug,
    {
        self.compiling_chunk.write(
            byte,
            self.parser
                .previous
                .clone()
                .expect("expected previous chunk")
                .line,
        );
    }

    fn emit_bytes<T, U>(&mut self, byte1: T, byte2: U)
    where
        T: Into<u8> + std::fmt::Debug,
        U: Into<u8> + std::fmt::Debug,
    {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) -> Result<()> {
        let constant = self.compiling_chunk.add_constant(value)?;

        self.emit_bytes(OpCode::Constant, constant);

        Ok(())
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        let _ = self.advance();

        let prefix_rule = self.get_rule(&self.parser.previous.clone().unwrap().token_type);

        match prefix_rule.prefix {
            ParseFn::None => {
                self.error("expected expression");
            }
            ParseFn::Number => self.number(),
            ParseFn::Binary => self.binary(),
            ParseFn::Unary => self.unary(),
            ParseFn::Grouping => self.grouping(),
        }

        loop {
            let current_rule = self.get_rule(&self.parser.current.clone().unwrap().token_type);

            if precedence > current_rule.precedence {
                break;
            }

            let _ = self.advance();

            match current_rule.infix {
                ParseFn::None => {
                    self.error("expected expression");
                }
                ParseFn::Number => self.number(),
                ParseFn::Binary => self.binary(),
                ParseFn::Unary => self.unary(),
                ParseFn::Grouping => self.grouping(),
            }
        }
    }

    fn get_rule(&self, tt: &TokenType) -> ParseRule {
        parse::parse_rule(tt)
    }
}

pub fn compile(source: String) -> Result<Chunk> {
    let mut compiler = Compiler::new(source);
    compiler.advance()?;
    compiler.expression();
    compiler.consume(TokenType::Eof, "Expected end of expression")?;
    compiler.emit_return();

    Ok(compiler.compiling_chunk)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn basic() {
        let source = String::from("1");
        let chunk = compile(source).unwrap();

        assert_eq!(vec![1, 0, 0], chunk.code);

        let source = String::from("-12");
        let chunk = compile(source).unwrap();

        assert_eq!(vec![1, 0, 2, 0], chunk.code);
    }
    #[test]
    fn arithmatic() {
        let source = String::from("1 + 2");
        let chunk = compile(source).unwrap();

        assert_eq!(vec![1, 0, 1, 1, 3, 0], chunk.code);

        let source = String::from("-1 + 2");
        let chunk = compile(source).unwrap();

        assert_eq!(vec![1, 0, 2, 1, 1, 3, 0], chunk.code);

        let source = String::from("(-1 + 2) * 3 - -4");
        let chunk = compile(source).unwrap();

        assert_eq!(vec![1, 0, 2, 1, 1, 3, 1, 2, 5, 1, 3, 2, 4, 0], chunk.code);
    }
}
