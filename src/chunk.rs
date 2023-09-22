use anyhow::{anyhow, Result};

use crate::error::{ChunkError, EvaluationError};

use std::ops::{Add, Div, Mul, Neg, Not, Sub};

const MAX_CONSTANTS: u8 = u8::MAX;

// TODO: Move to module
#[derive(Debug)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
    Nil,
    True,
    False,
    Negate,
    Not,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    Greater,
    Less,
}

impl From<OpCode> for u8 {
    fn from(o: OpCode) -> u8 {
        o as u8
    }
}

impl TryFrom<u8> for OpCode {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Return),
            1 => Ok(OpCode::Constant),
            2 => Ok(OpCode::Nil),
            3 => Ok(OpCode::True),
            4 => Ok(OpCode::False),
            5 => Ok(OpCode::Negate),
            6 => Ok(OpCode::Not),
            7 => Ok(OpCode::Add),
            8 => Ok(OpCode::Subtract),
            9 => Ok(OpCode::Multiply),
            10 => Ok(OpCode::Divide),
            11 => Ok(OpCode::Equal),
            12 => Ok(OpCode::Greater),
            13 => Ok(OpCode::Less),
            n => Err(ChunkError::UnknownOpCode(n).into()),
        }
    }
}

pub struct Chunk {
    pub code: Vec<u8>,
    constants: Array<Value>,
    lines: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Nil => true,
            Value::Bool(v) => !v,
            _ => false,
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Number(ref n) => write!(f, "{}", n),
            Self::Bool(ref b) => write!(f, "{}", b),
            Self::Nil => write!(f, "nil"),
        }
    }
}

impl Add for Value {
    type Output = Result<Value>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a + b)),
            // (Self::String(a), Self::String(b)) => Ok(Self::String(a + &b)),
            // (Self::String(_), _) | (_, Self::String(_)) => {
            //     Err(EvaluationError::StringConcatination.into())
            // }
            // (Self::Number(a), Self::Nil) | (Self::Nil, Self::Number(a)) => Ok(Self::Number(a)), // nil -> 0 in Lox
            (_, _) => Err(EvaluationError::Arithmatic("add".to_string()).into()),
        }
    }
}

impl Sub for Value {
    type Output = Result<Value>;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a - b)),
            (_, _) => Err(EvaluationError::Arithmatic("subtract".to_string()).into()),
        }
    }
}

impl Mul for Value {
    type Output = Result<Value>;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a * b)),
            (_, _) => Err(EvaluationError::Arithmatic("multiply".to_string()).into()),
        }
    }
}

impl Div for Value {
    type Output = Result<Value>;
    fn div(self, rhs: Value) -> Self::Output {
        match (self, rhs) {
            (Self::Number(a), Self::Number(b)) => Ok(Self::Number(a / b)),
            (_, _) => Err(EvaluationError::Arithmatic("divide".to_string()).into()),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value>;

    fn neg(self) -> Self::Output {
        match self {
            Self::Number(a) => Ok(Self::Number(a.neg())),
            _ => Err(EvaluationError::Negation.into()),
        }
    }
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        match self {
            Self::Bool(a) => Self::Output::Bool(!a),
            Self::Nil => Self::Output::Bool(false),
            _ => Self::Output::Bool(true),
        }
    }
}

impl Not for &Value {
    type Output = Value;
    fn not(self) -> Self::Output {
        match *self {
            Value::Bool(a) => Self::Output::Bool(!a),
            Value::Nil => Self::Output::Bool(true),
            _ => Self::Output::Bool(false),
        }
    }
}

#[derive(Debug)]
pub struct Array<T> {
    values: Vec<T>,
}

impl<T> Array<T> {
    pub fn new() -> Array<T> {
        Array { values: Vec::new() }
    }

    pub fn write(&mut self, value: T) {
        self.values.push(value);
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Array::new(),
            lines: Vec::new(),
        }
    }

    pub fn write<T, U>(&mut self, byte: T, line: U)
    where
        T: Into<u8>,
        U: Into<usize>,
    {
        self.code.push(byte.into());
        self.lines.push(line.into());
    }

    // TODO: value: dyn Into<Value>
    pub fn add_constant(&mut self, value: Value) -> Result<u8> {
        if self.constants.len() >= MAX_CONSTANTS as usize {
            return Err(anyhow!("too many constants in this chunk"));
        }
        self.constants.write(value);
        Ok(self.constants.values.len() as u8 - 1)
    }

    pub fn read_constant(&self, loc: usize) -> Value {
        self.constants.values[loc].clone()
    }

    pub fn disassemble(&self, header: &str) {
        println!("== {} ==", header);
        let mut offset = 0;

        // TODO: Iterator for this
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        let mut offset = offset;
        print!("{:0>4} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:>4} ", self.lines[offset]);
        }

        let instruction = self.code[offset];
        let output = match instruction.try_into() {
            Ok(OpCode::Return) => {
                offset += 1;
                format!("{}", "OP_RETURN")
            }
            Ok(OpCode::Negate) => {
                offset += 1;
                format!("{}", "OP_NEGATE")
            }
            Ok(OpCode::Add) => {
                offset += 1;
                format!("{}", "OP_ADD")
            }
            Ok(OpCode::Subtract) => {
                offset += 1;
                format!("{}", "OP_SUBTRACT")
            }
            Ok(OpCode::Multiply) => {
                offset += 1;
                format!("{}", "OP_MULTIPLY")
            }
            Ok(OpCode::Divide) => {
                offset += 1;
                format!("{}", "OP_DIVIDE")
            }
            Ok(OpCode::Constant) => {
                let constant = &self.code[offset + 1];
                offset += 2;
                format!(
                    "{:<16} {:>4} '{}'",
                    "OP_CONSTANT", constant, self.constants.values[*constant as usize]
                )
            }
            Ok(OpCode::Nil) => {
                offset += 1;
                format!("{}", "OP_NIL")
            }
            Ok(OpCode::True) => {
                offset += 1;
                format!("{}", "OP_TRUE")
            }
            Ok(OpCode::False) => {
                offset += 1;
                format!("{}", "OP_FALSE")
            }
            Ok(OpCode::Not) => {
                offset += 1;
                format!("{}", "OP_NOT")
            }
            Ok(OpCode::Equal) => {
                todo!()
            }
            Ok(OpCode::Greater) => {
                todo!()
            }
            Ok(OpCode::Less) => {
                todo!()
            }
            Err(_) => format!("unknown opcode {}", instruction),
        };

        println!("{}", output);

        offset
    }
}
