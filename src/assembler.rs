use std::io::Write;
use crate::{assembler::reader::Reader, instruction::{Address, Instruction}};

mod reader;

pub type AssemblySource<'a> = &'a [u8];

pub fn parse_instructions(source: AssemblySource, program_address: Address) -> Option<Vec<Instruction>> {
    let mut reader = Reader::new(source);
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
