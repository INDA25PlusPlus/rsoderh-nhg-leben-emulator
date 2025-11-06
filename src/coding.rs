use std::io::Write;

use crate::{coding::reader::Reader, instruction::Instruction};

mod reader;

pub fn encode(buffer: impl Write, instructions: Vec<Instruction>) -> std::io::Result<()> {
    todo!()
}

pub fn decode<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    
    todo!()
}

