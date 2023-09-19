use crate::chunk::{Chunk, OpCode, Value};

use crate::LOX_TRACE_EXECUTION;

const STACK_MAX: u32 = 256;

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

pub enum InterpretError {
    Compile,
    Runtime,
}

impl<'a> VM<'a> {
    pub fn interpret(chunk: &'a Chunk) -> Result<(), InterpretError> {
        let mut vm = VM {
            chunk,
            ip: 0,
            stack: Vec::with_capacity(STACK_MAX as usize),
        };

        vm.run()
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            if LOX_TRACE_EXECUTION.get() == Some(&true) {
                print!("          ");
                for item in &self.stack {
                    print!("[ {} ]", item);
                }
                println!("");
                let _ = self.chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.chunk.code[self.ip];
            self.ip += 1;

            match instruction.try_into() {
                Ok(OpCode::Return) => {
                    if let Some(v) = self.stack.pop() {
                        println!("{}", v)
                    }
                    break;
                }
                Ok(OpCode::Negate) => {
                    if let Some(value) = self.stack.pop() {
                        self.stack.push(-value);
                    }
                }
                Ok(OpCode::Constant) => {
                    let constant = self.chunk.read_constant(self.chunk.code[self.ip] as usize);
                    self.ip += 1;
                    self.stack.push(constant);
                }
                Err(_) => return Err(InterpretError::Runtime),
            }
        }

        Ok(())
    }
}
