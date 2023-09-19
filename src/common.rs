use std::ops::Deref;

enum OpCode {
    Return,
}

struct Chunk(Vec<u8>);

impl Deref for Chunk {
    type Target = Vec<u8>;
    fn deref(&self) -> &Vec<u8> {
        &self.0
    }
}
