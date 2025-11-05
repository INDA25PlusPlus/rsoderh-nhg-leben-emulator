use crate::{assembler::reader::{ReadError, ReadResult, Reader}, instruction::{Address, Instruction}};

pub type Label<'a> = &'a [u8];
