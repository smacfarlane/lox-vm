use crate::chunk::{Chunk, OpCode, Value};
use crate::error::InterpretError;
use crate::LOX_TRACE_EXECUTION;

use anyhow::Result;

const STACK_MAX: u32 = 256;

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl<'a> VM<'a> {
    pub fn interpret(source: String) -> Result<()> {
        let chunk = crate::compiler::compile(source).map_err(|_| InterpretError::Compile)?;

        let mut vm = VM {
            chunk: &chunk,
            ip: 0,
            stack: Vec::with_capacity(STACK_MAX as usize), // TODO: This is a "soft max"
        };

        vm.run()
    }

    fn runtime_error(&mut self) -> Result<()> {
        Err(InterpretError::Runtime.into())
    }

    pub fn run(&mut self) -> Result<()> {
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

            let opcode = instruction.try_into()?;
            match opcode {
                OpCode::Return => {
                    if let Some(v) = self.stack.pop() {
                        println!("{}", v)
                    }
                    break;
                }
                OpCode::Negate => {
                    if let Some(value) = self.stack.pop() {
                        match -value {
                            Ok(value) => self.stack.push(value),
                            Err(_) => self.runtime_error()?,
                        }
                    }
                }
                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match a + b {
                        Ok(sum) => self.stack.push(sum),
                        Err(_) => self.runtime_error()?,
                    }
                }
                OpCode::Subtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match a - b {
                        Ok(diff) => self.stack.push(diff),
                        Err(_) => self.runtime_error()?,
                    }
                }
                OpCode::Multiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match a * b {
                        Ok(prod) => self.stack.push(prod),
                        Err(_) => self.runtime_error()?,
                    }
                }
                OpCode::Divide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    match a / b {
                        Ok(quot) => self.stack.push(quot),
                        Err(_) => self.runtime_error()?,
                    }
                }
                OpCode::Constant => {
                    let constant = self.chunk.read_constant(self.chunk.code[self.ip] as usize);
                    self.ip += 1;
                    self.stack.push(constant);
                }
                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                }
                OpCode::True => {
                    self.stack.push(Value::Bool(true));
                }
                OpCode::False => {
                    self.stack.push(Value::Bool(false));
                }
                OpCode::Not => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(value.is_falsey()))
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                }
                OpCode::Greater => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a > b));
                }
                OpCode::Less => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a < b));
                }
            }
        }

        Ok(())
    }
}
