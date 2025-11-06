pub mod instruction;
mod number;
mod token;

use parsable::{CharLiteral, Ignore, Parsable, ZeroPlus};

use crate::{assembler::reader::{ReadError, ReadResult, Reader}, instruction::{Address, Instruction}};

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub struct SourceFile {

}

pub type Ws = Ignore<ZeroPlus<WsChar>>;

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
enum WsChar {
    Space(CharLiteral<b' '>),
    Tab(CharLiteral<b'\t'>),
    Crlf(#[literal = b"\r\n"] ()),
    Cr(CharLiteral<b'\r'>),
    Lf(CharLiteral<b'\n'>),
}
