mod chunk;
mod vm;

use crate::chunk::{Chunk, OpCode};
use std::env;
use std::sync::OnceLock;

const LOX_TRACE_EXECUTION_VAR: &str = "LOX_TRACE_EXECUTION";
static LOX_TRACE_EXECUTION: OnceLock<bool> = OnceLock::new();

fn main() {
    let _ = LOX_TRACE_EXECUTION.set(env::var(LOX_TRACE_EXECUTION_VAR).is_ok());
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::Constant, 1);
    chunk.write(constant, 1);
    chunk.write(OpCode::Negate, 1);
    chunk.write(OpCode::Return, 1);

    // chunk.disassemble("test");

    match crate::vm::VM::interpret(&chunk) {
        Ok(()) => {
            println!("execution finished successfully")
        }
        Err(_e) => {
            println!("error in execution")
        }
    }
}
