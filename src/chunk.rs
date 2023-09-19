// TODO: Move to module
#[repr(u8)]
pub enum OpCode {
    Return,
    Constant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl From<OpCode> for u8 {
    fn from(o: OpCode) -> u8 {
        o as u8
    }
}

pub enum Error {
    UnknownOpCode,
}

impl TryFrom<u8> for OpCode {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OpCode::Return),
            1 => Ok(OpCode::Constant),
            2 => Ok(OpCode::Negate),
            3 => Ok(OpCode::Add),
            4 => Ok(OpCode::Subtract),
            5 => Ok(OpCode::Multiply),
            6 => Ok(OpCode::Divide),
            _ => Err(Error::UnknownOpCode),
        }
    }
}

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<u8>,
    constants: Array<Value>,
    lines: Vec<u8>,
}

pub type Value = f64;
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
        U: Into<u8>,
    {
        self.code.push(byte.into());
        self.lines.push(line.into());
    }

    // TODO: value: dyn Into<Value>
    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write(value);
        self.constants.values.len() as u8 - 1 // TODO: unsafe
    }

    pub fn read_constant(&self, loc: usize) -> Value {
        self.constants.values[loc]
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
            Err(_) => format!("unknown opcode {}", instruction),
        };

        println!("{}", output);

        offset
    }
}
