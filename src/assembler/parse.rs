pub mod instruction;
mod number;
mod token;

use std::fmt::Debug;
use parsable::{CharLiteral, CharRange, EndOfStream, Ignore, Parsable, WithIndex, ZeroPlus};

use crate::assembler::{labels::Label, parse::{instruction::ParsedInstruction, number::LiteralNumber, token::{Colon, EndOfAssembly, Origin, Semicolon}}};

#[derive(Clone, PartialEq, Eq, Parsable)]
pub struct SourceFile {
    _0: WsNl,
    comments: ZeroPlus<CommentOnlyLine>,
    pub origin_line: Option<OriginLine>,
    pub lines: ZeroPlus<CodeLine>,
    end: EndOfAssemblyLine,
}

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub struct CommentOnlyLine(CommentSegment, WsNl);

#[derive(Clone, PartialEq, Eq, Parsable)]
pub struct OriginLine {
    pub label: Option<LabelSegment>,
    keyword: Origin,
    _0: Ws,
    pub address: WithIndex<LiteralNumber>,
    _1: WsNl,
}

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub struct CodeLine {
    pub label: Option<LabelSegment>,
    pub code: Option<CodeSegment>,
    comment: Option<CommentSegment>,
    _0: Ws,
    _1: Nl,
}

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub struct EndOfAssemblyLine(Option<LabelSegment>, EndOfAssembly, WsNl, EndOfStream);

#[derive(Clone, PartialEq, Eq, Parsable)]
pub struct LabelSegment(pub WithIndex<Label>, Colon, Ws);

impl Debug for LabelSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LabelSegment").field(&self.0.node).field(&self.1).field(&self.2).finish()
    }
}

#[derive(Clone, PartialEq, Eq, Parsable)]
pub struct CodeSegment {
    pub instruction: WithIndex<ParsedInstruction>,
    _0: Ws,
}

impl Debug for CodeSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeSegment").field("instruction", &self.instruction.node).field("_0", &self._0).finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub struct CommentSegment(Semicolon, ZeroPlus<NonNlChar>);

pub type WsNl = Ignore<ZeroPlus<WsNlChar>>;

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub enum WsNlChar {
    Ws(WsChar),
    Nl(Nl),
}

pub type Ws = Ignore<ZeroPlus<WsChar>>;

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub enum WsChar {
    Space(CharLiteral<b' '>),
    Tab(CharLiteral<b'\t'>),
}

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub enum Nl {
    Crlf(#[literal = b"\r\n"] ()),
    Cr(CharLiteral<b'\r'>),
    Lf(CharLiteral<b'\n'>),
}

pub type NonNlChar = Ignore<NonNlCharInner>;

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub enum NonNlCharInner {
    Tab(CharLiteral<b'\t'>),
    Other(CharRange<b' ', b'~'>),
}
