use parsable::Parsable;
use std::io::Write;

use crate::{
    assembler::parse::SourceFile,
    instruction::{Address, Instruction},
};

mod labels;
mod parse;

pub type AssemblySource<'a> = &'a [u8];

pub fn parse_instructions(
    source: AssemblySource,
    program_address: Address,
) -> Option<Vec<Instruction>> {
    let mut stream = parsable::ScopedStream::new(source);
    let outcome = parsable::WithEnd::<SourceFile>::parse(&mut stream);

    loop {
        todo!()
    }
}
