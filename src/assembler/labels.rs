use std::collections::HashMap;

use parsable::{CharLiteral, CharRange, Parsable, Span, ZeroPlus};

use crate::instruction::Address;

pub struct LabelLookup {
    map: HashMap<Vec<u8>, Address>,
}

impl LabelLookup {
    pub fn new() -> LabelLookup {
        LabelLookup {
            map: HashMap::new(),
        }
    }

    fn to_label_ident(label: &Label) -> Vec<u8> {
        label.span[..label.span.len().max(5)].to_owned()
    }

    pub fn insert(&mut self, label: Label, address: Address) -> Result<(), ()> {
        let ident = LabelLookup::to_label_ident(&label);
        if self.map.contains_key(&ident) {
            Err(())
        } else {
            self.map.insert(ident, address);
            Ok(())
        }
    }

    pub fn get(&self, label: Label) -> Option<Address> {
        let ident = LabelLookup::to_label_ident(&label);
        self.map.get(&ident).copied()
    }
}

pub type Label = Span<LabelInner>;

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
pub struct LabelInner {
    initial: InitialLabelChar,
    rest: ZeroPlus<LabelChar>,
}

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
enum InitialLabelChar {
    At(CharLiteral<b'@'>),
    QuestionMark(CharLiteral<b'?'>),
    Alpha(CharRange<b'A', b'Z'>),
}

#[derive(Clone, Debug, PartialEq, Eq, Parsable)]
enum LabelChar {
    Alpha(CharRange<b'A', b'Z'>),
    Numerical(CharRange<b'0', b'9'>),
}