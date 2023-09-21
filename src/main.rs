mod chunk;
mod compiler;
mod error;
mod parse;
mod scanner;
mod token;
mod vm;

use crate::chunk::{Chunk, OpCode};
use std::env;
use std::sync::OnceLock;

const LOX_TRACE_EXECUTION_VAR: &str = "LOX_TRACE_EXECUTION";
static LOX_TRACE_EXECUTION: OnceLock<bool> = OnceLock::new();

fn main() {
    let _ = LOX_TRACE_EXECUTION.set(env::var(LOX_TRACE_EXECUTION_VAR).is_ok());

    let source = String::from("(-1 + 2) * 3 - -4");
    match crate::vm::VM::interpret(source) {
        Ok(()) => {
            println!("execution finished successfully")
        }
        Err(_e) => {
            println!("error in execution")
        }
    }
}
