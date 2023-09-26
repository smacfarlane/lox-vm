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

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;

        loop {
            match self.parser.current.clone().unwrap().token_type {
                TokenType::Eof
                | TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            if self.parser.previous.clone().unwrap().token_type == TokenType::Semicolon {
                return;
            }

            let _ = self.advance();
        }
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

    fn variable(&mut self, can_assign: bool) {
        self.named_variable(self.parser.previous.clone().unwrap(), can_assign);
    }

    fn named_variable(&mut self, name: Token, can_assign: bool) {
        let arg = Value::from_string(name.lexeme);
        let constant = self.compiling_chunk.add_constant(arg).unwrap();
        if can_assign && self.current_token_type_is(TokenType::Equal) {
            self.expression();
            self.emit_bytes(OpCode::SetGlobal, constant);
        } else {
            self.emit_bytes(OpCode::GetGlobal, constant);
        }
    }

    fn number(&mut self, can_assign: bool) {
        let value = self
            .parser
            .previous
            .clone()
            .expect("expected previous chunk")
            .lexeme;
        let value: f64 = value
            .parse()
            .expect(&format!("unable to convert token to float {}", value));

        let _ = self.emit_constant(Value::Number(value));
    }

    fn string(&mut self, can_assign: bool) {
        let value = self
            .parser
            .previous
            .clone()
            .expect("expected previous chunk")
            .lexeme;
        // Strip "" from the Token representation
        let value = &value[1..value.len() - 1];

        let value = Value::from_string(value.to_string());
        let _ = self.emit_constant(value);
    }

    fn literal(&mut self, can_assign: bool) {
        let tt = self
            .parser
            .previous
            .clone()
            .expect("expected previous chunk")
            .token_type;

        match tt {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            TokenType::True => self.emit_byte(OpCode::True),
            _ => unreachable!(),
        }
    }

    fn current_token_type_is(&mut self, tt: TokenType) -> bool {
        let current_tt = self
            .parser
            .current
            .clone()
            .expect("expected current token")
            .token_type;
        if current_tt == tt {
            let _ = self.advance();
            true
        } else {
            false
        }
    }

    fn declaration(&mut self) {
        if self.current_token_type_is(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable().unwrap(); // TODO: Handle this

        if self.current_token_type_is(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "expected ';' after variable declaration",
        );
        self.define_variable(global);
    }

    fn parse_variable(&mut self) -> Result<u8> {
        self.consume(TokenType::Identifier, "expected variable name")?;
        let value = self.parser.previous.clone().unwrap().lexeme;
        self.compiling_chunk.add_constant(Value::from_string(value))
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(OpCode::DefineGlobal, global);
    }

    fn statement(&mut self) {
        if self.current_token_type_is(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "expect ';' after value.");
        self.emit_byte(OpCode::Pop);
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        let _ = self.consume(TokenType::RightParen, "expected ')' after expression)");
    }

    fn unary(&mut self, can_assign: bool) {
        let operator_type = self
            .parser
            .previous
            .clone()
            .expect("expected previous token")
            .token_type;

        self.parse_precedence(Precedence::Unary);

        match operator_type {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            TokenType::Bang => self.emit_byte(OpCode::Not),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self, can_assign: bool) {
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
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal, OpCode::Not),
            TokenType::Equal => self.emit_byte(OpCode::Equal),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less, OpCode::Not),
            TokenType::Less => self.emit_byte(OpCode::Equal),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater, OpCode::Not),
            _ => {
                dbg!(operator_type);
                unreachable!()
            }
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

        let can_assign = precedence <= Precedence::Assignment;

        match prefix_rule.prefix {
            ParseFn::None => {
                self.error("expected expression");
            }
            ParseFn::Number => self.number(can_assign),
            ParseFn::Literal => self.literal(can_assign),
            ParseFn::String => self.string(can_assign),
            ParseFn::Variable => self.variable(can_assign),
            ParseFn::Binary => self.binary(can_assign),
            ParseFn::Unary => self.unary(can_assign),
            ParseFn::Grouping => self.grouping(can_assign),
        }

        if can_assign && self.current_token_type_is(TokenType::Equal) {
            self.error("invalid assignment target");
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
                ParseFn::Number => self.number(can_assign),
                ParseFn::Literal => self.literal(can_assign),
                ParseFn::String => self.string(can_assign),
                ParseFn::Variable => self.variable(can_assign),
                ParseFn::Binary => self.binary(can_assign),
                ParseFn::Unary => self.unary(can_assign),
                ParseFn::Grouping => self.grouping(can_assign),
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

    loop {
        if compiler.current_token_type_is(TokenType::Eof) {
            break;
        }
        compiler.declaration();
    }

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

        assert_eq!(vec![1, 0, 15, 0], chunk.code);

        let source = String::from("-12");
        let chunk = compile(source).unwrap();

        assert_eq!(vec![1, 0, 5, 15, 0], chunk.code);
    }
    #[test]
    fn arithmatic() {
        let source = String::from("1 + 2");
        let chunk = compile(source).unwrap();

        assert_eq!(vec![1, 0, 1, 1, 7, 15, 0], chunk.code);

        let source = String::from("-1 + 2");
        let chunk = compile(source).unwrap();

        assert_eq!(vec![1, 0, 5, 1, 1, 7, 15, 0], chunk.code);

        let source = String::from("(-1 + 2) * 3 - -4");
        let chunk = compile(source).unwrap();

        assert_eq!(
            vec![1, 0, 5, 1, 1, 7, 1, 2, 9, 1, 3, 5, 8, 15, 0],
            chunk.code
        );
    }

    #[test]
    fn logic() {
        let source = String::from("!(5 - 4 > 3 * 2 == !nil)");

        let chunk = compile(source).unwrap();

        assert_eq!(
            vec![1, 0, 1, 1, 8, 1, 2, 1, 3, 9, 12, 2, 6, 11, 6, 15, 0],
            chunk.code
        );
    }
}
