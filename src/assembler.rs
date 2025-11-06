use parsable::{Parsable, format_error_stack};
use std::io::Write;

use crate::{
    assembler::{labels::{Label, LabelLookup}, parse::{LabelSegment, SourceFile}},
    instruction::{Address, InstructionOrData},
};

mod labels;
mod parse;

pub type AssemblySource<'a> = &'a [u8];

pub fn parse_assembly(
    source: AssemblySource,
) -> Result<(Vec<InstructionOrData>, u16), String> {
    let mut stream = parsable::ScopedStream::new(source);
    let outcome = parsable::WithEnd::<SourceFile>::parse(&mut stream);
    let source_file = match outcome.expect("parsing should give a result") {
        Ok(parsed) => parsed.node,
        Err(stack) => return Err(format_error_stack(source, stack)),
    };
    
    let origin_address: Address = if let Some(origin_line) = &source_file.origin_line {
        origin_line.address.node.clone().try_into()
            .map_err(|_| format!("{}: Expected address", origin_line.address.index))?
    } else {
        0x0000_0000
    };

    let mut labels = LabelLookup::new();
    let mut add_label = |source_pos: usize, label: Label, address: u16| {
        // this is kind of inefficient but i couldn't find a better way to do it
        labels.insert(label.clone(), address).map_err(|_|
            format!("{}: Duplicate label {}", source_pos, String::from_utf8_lossy(&label.span)))
    };
    let mut add_label_segment_opt = |label_segment: Option<&LabelSegment>, address: u16| {
        if let Some(label_segment) = label_segment {
            add_label(label_segment.0.index, label_segment.0.node.clone(), address)
        } else {
            Ok(())
        }
    };

    let mut current_address = origin_address;

    add_label_segment_opt(
        source_file.origin_line.as_ref().map(|origin_line| origin_line.label.as_ref()).flatten(),
        current_address,
    )?;
    
    for code_line in &source_file.lines.nodes {
        add_label_segment_opt(code_line.label.as_ref(), current_address)?;
        if let Some(code) = &code_line.code {
            let instruction = &code.instruction;
            current_address = current_address.checked_add(instruction.node.instruction_length())
                .ok_or(format!("{}: Memory size overflowed", instruction.index))?;
        }
    }

    let mut instructions = Vec::new();
    for code_line in source_file.lines.nodes {
        if let Some(code) = code_line.code {
            let instruction = code.instruction.node.into_inner(&labels)
                .ok_or(format!("{}: Unknown label", code.instruction.index))?;
            instructions.push(InstructionOrData::Instruction(instruction));
        }
    }
    Ok((instructions, origin_address))
}
