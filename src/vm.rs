use crate::chunk::{Chunk, OpCode, Value};
use crate::error::{InterpretError, RuntimeError};
use crate::LOX_TRACE_EXECUTION;

use anyhow::Result;

use std::collections::HashMap;

const STACK_MAX: u32 = 256;

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl<'a> VM<'a> {
    pub fn interpret(source: String) -> Result<()> {
        let chunk = crate::compiler::compile(source).map_err(|_| InterpretError::Compile)?;

        let mut vm = VM {
            chunk: &chunk,
            ip: 0,
            stack: Vec::with_capacity(STACK_MAX as usize), // TODO: This is a "soft max"
            globals: HashMap::new(),
        };

        vm.run()
    }

    fn runtime_error(&mut self) -> Result<()> {
        Err(InterpretError::Runtime.into())
    }

    pub fn run(&mut self) -> Result<()> {
        self.chunk.disassemble("RUN");
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

            match instruction.try_into()? {
                OpCode::Return => return Ok(()),
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
                OpCode::Print => {
                    let a = self.stack.pop().unwrap();
                    println!("{}", a);
                }
                OpCode::Pop => {
                    let _ = self.stack.pop();
                }
                OpCode::DefineGlobal => {
                    let name = self.chunk.read_constant(self.chunk.code[self.ip] as usize);
                    self.ip += 1;
                    self.globals
                        .insert(name.to_string(), self.stack.last().unwrap().to_owned());

                    let _ = self.stack.pop();
                }
                OpCode::GetGlobal => {
                    let name = self.chunk.read_constant(self.chunk.code[self.ip] as usize);
                    self.ip += 1;
                    match self.globals.get(&name.to_string()) {
                        Some(value) => self.stack.push(value.to_owned()),
                        None => self.runtime_error()?,
                    }
                }
                OpCode::SetGlobal => {
                    let name = self.chunk.read_constant(self.chunk.code[self.ip] as usize);
                    self.ip += 1;

                    if !self.globals.contains_key(&name.to_string()) {
                        self.runtime_error()?
                    }

                    self.globals
                        .insert(name.to_string(), self.stack.last().unwrap().to_owned());

                    let _ = self.stack.pop();
                }
            }
        }
    }
}
