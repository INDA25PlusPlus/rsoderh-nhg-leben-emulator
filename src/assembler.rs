use std::io::Write;
use parsable::Parsable;

use crate::{assembler::{parse::SourceFile, reader::Reader}, instruction::{Address, Instruction}};

mod labels;
mod parse;
mod reader;

pub type AssemblySource<'a> = &'a [u8];

pub fn parse_instructions(source: AssemblySource, program_address: Address) -> Option<Vec<Instruction>> {
    let mut stream = parsable::ScopedStream::new(source);
    let outcome = parsable::WithEnd::<SourceFile>::parse(&mut stream);
    
    loop {
        todo!()
    }
}

pub fn encode(buffer: impl Write, instructions: Vec<Instruction>) -> std::io::Result<()> {
    todo!()
}

pub fn decode(buffer: &[u8]) -> Option<Vec<Instruction>> {
    todo!()
}
