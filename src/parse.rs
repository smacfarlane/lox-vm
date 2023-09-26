use crate::token::{Token, TokenType};

#[derive(PartialOrd, PartialEq, Debug)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    // TODO: Is there a better way to increment the priority?
    pub fn next(&self) -> Precedence {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary, // TODO: This is incorrect
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    pub current: Option<Token>,
    pub previous: Option<Token>,
    pub had_error: bool,
    pub panic_mode: bool,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
        }
    }

    pub fn advance(&mut self) {
        self.previous = self.current.clone();
        self.current = None;
    }
}

#[derive(Debug)]
pub enum ParseFn {
    Binary,
    Grouping,
    Unary,
    Number,
    Literal,
    String,
    None,
}

#[derive(Debug)]
pub struct ParseRule {
    pub prefix: ParseFn,
    pub infix: ParseFn,
    pub precedence: Precedence,
}

// TODO: Figure out how to make this into a lookup table as in the book
// Benchmark: Is it faster?
pub fn parse_rule(tt: &TokenType) -> ParseRule {
    match tt {
        TokenType::LeftParen => ParseRule {
            prefix: ParseFn::Grouping,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::RightParen => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::LeftBrace => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::RightBrace => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Comma => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Dot => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Minus => ParseRule {
            prefix: ParseFn::Unary,
            infix: ParseFn::Binary,
            precedence: Precedence::Term,
        },
        TokenType::Plus => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Term,
        },
        TokenType::Semicolon => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Slash => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Factor,
        },
        TokenType::Star => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Factor,
        },
        TokenType::Bang => ParseRule {
            prefix: ParseFn::Unary,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::BangEqual => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Equality,
        },
        TokenType::Equal => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::EqualEqual => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Equality,
        },
        TokenType::Greater => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Comparison,
        },
        TokenType::GreaterEqual => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Comparison,
        },
        TokenType::Less => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Comparison,
        },
        TokenType::LessEqual => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::Binary,
            precedence: Precedence::Comparison,
        },
        TokenType::Identifier => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::String => ParseRule {
            prefix: ParseFn::String,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Number => ParseRule {
            prefix: ParseFn::Number,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::And => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Class => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Else => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::False => ParseRule {
            prefix: ParseFn::Literal,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::For => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Fun => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::If => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Nil => ParseRule {
            prefix: ParseFn::Literal,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Or => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Print => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Return => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Super => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::This => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::True => ParseRule {
            prefix: ParseFn::Literal,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Var => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::While => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
        TokenType::Eof => ParseRule {
            prefix: ParseFn::None,
            infix: ParseFn::None,
            precedence: Precedence::None,
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn precedence() {
        assert!(Precedence::Assignment <= Precedence::Term);
    }
}
