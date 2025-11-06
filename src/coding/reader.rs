use crate::instruction::{Register, RegisterPair};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ReadError<'c> {
    EndOfBuffer,
    UnexpectedChar(u8),
    UnexpectedSlice(&'c [u8]),
}

type Label<'a> = &'a [u8];

pub type ReadResult<'c, T> = Result<T, ReadError<'c>>;

macro_rules! parse_alternatives {
    ($reader:expr, $($expected:literal, $result:expr,)*) => {
        Err(ReadError::EndOfBuffer)
            $( .or_else(|_| $reader.expect_slice($expected).map(|_| $result)) )*
    };
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Reader<'a> {
    original: &'a [u8],
    buffer: &'a [u8],
}

impl<'a> Reader<'a> {
    pub fn new(slice: &[u8]) -> Reader {
        Reader {
            original: slice,
            buffer: slice,
        }
    }

    pub fn read(&mut self) -> Option<u8> {
        self.read_n(1).map(|slice| slice[0])
    }

    pub fn read_n<'b>(&'b mut self, n: usize) -> Option<&'a [u8]> {
        if self.buffer.len() < n {
            return None;
        }
        let (ret, rest) = self.buffer.split_at(n);
        self.buffer = rest;
        Some(ret)
    }

    pub fn read_pred<'b>(&'b mut self, pred: impl FnOnce(u8) -> bool) -> Option<u8> {
        if let Some(value) = self.peek() {
            if pred(value) {
                self.skip();
                return Some(value);
            }
        }
        None
    }

    pub fn skip(&mut self) {
        let _ = self.read();
    }

    pub fn skip_n(&mut self, n: usize) {
        let _ = self.read_n(n);
    }

    pub fn peek(&self) -> Option<u8> {
        self.peek_n(1).map(|slice| slice[0])
    }

    pub fn peek_at(&self, index: usize) -> Option<u8> {
        if self.buffer.len() <= index {
            return None;
        }
        Some(self.buffer[index])
    }

    pub fn peek_n<'b>(&'b self, n: usize) -> Option<&'a [u8]> {
        if self.buffer.len() < n {
            return None;
        }
        Some(&self.buffer[..n])
    }

    pub fn read_until<'b>(&'b mut self, value: u8) -> Option<&'a [u8]> {
        for i in 0..self.buffer.len() {
            if self.peek_at(i).unwrap() == value {
                let ret = self.read_n(i).unwrap();
                self.skip();
                return Some(ret);
            }
        }
        None
    }

    pub fn read_until_or_end<'b>(&'b mut self, value: u8) -> &'a [u8] {
        self.read_until(value)
            .unwrap_or_else(|| self.read_n(self.buffer.len()).unwrap())
    }

    pub fn at_end(&self) -> bool {
        self.buffer.len() == 0
    }

    pub fn skip_ws(&mut self) {
        fn is_ws(value: Option<u8>) -> bool {
            value.is_some_and(|value| value == b' ' || value == b'\t')
        }
        while is_ws(self.peek()) {
            self.skip();
        }
    }

    pub fn skip_ws_comment(&mut self) -> bool {
        loop {
            let v = match self.peek() {
                Some(v) => v,
                None => return false,
            };
            if v == b' ' || v == b'\t' {
                self.skip();
            } else if v == b';' {
                while matches!(self.read_pred(|v| v != b'\n' && v != b'\r'), Some(..)) {}
                return true;
            } else {
                return false;
            }
        }
    }

    pub fn skip_ws_nl_comments(&mut self) {
        while self.skip_ws_comment() {
            self.skip_ws_nl();
        }
    }

    pub fn skip_ws_nl(&mut self) {
        fn is_ws_nl(value: Option<u8>) -> bool {
            value.is_some_and(|value| {
                value == b' ' || value == b'\t' || value == b'\n' || value == b'\r'
            })
        }
        while is_ws_nl(self.peek()) {
            self.skip();
        }
    }

    pub fn expect<'b>(&'b mut self, expected: u8) -> ReadResult<'a, ()> {
        if let Some(value) = self.read() {
            if value == expected {
                Ok(())
            } else {
                Err(ReadError::UnexpectedChar(value))
            }
        } else {
            Err(ReadError::EndOfBuffer)
        }
    }

    pub fn expect_pred<'b>(&'b mut self, pred: impl FnOnce(u8) -> bool) -> ReadResult<'a, u8> {
        if let Some(value) = self.read() {
            if pred(value) {
                Ok(value)
            } else {
                Err(ReadError::UnexpectedChar(value))
            }
        } else {
            Err(ReadError::EndOfBuffer)
        }
    }

    pub fn expect_slice<'b>(&'b mut self, expected: &[u8]) -> ReadResult<'a, ()> {
        if let Some(value) = self.read_n(expected.len()) {
            if value == expected {
                Ok(())
            } else {
                Err(ReadError::UnexpectedSlice(value))
            }
        } else {
            Err(ReadError::EndOfBuffer)
        }
    }

    pub fn expect_nl<'b>(&'b mut self) -> ReadResult<'a, ()> {
        self.skip_ws();
        self.expect_slice(b"\r\n")
            .or_else(|_| self.expect(b'\n'))
            .or_else(|_| self.expect(b'\r'))?;
        self.skip_ws();
        Ok(())
    }

    pub fn expect_label_name<'b>(&'b mut self) -> ReadResult<'a, Label<'a>> {
        let is_alpha = |v| (b'a'..=b'z').contains(&v) || (b'A'..=b'Z').contains(&v);
        let is_num = |v| (b'0'..=b'9').contains(&v);

        let buf = self.buffer;
        self.expect_pred(|v| v == b'@' || v == b'?' || is_alpha(v))?;

        let mut len = 1;
        while matches!(self.read_pred(|v| is_alpha(v) || is_num(v)), Some(..)) {
            len += 1;
        }
        self.expect(b':')?;
        Ok(&buf[..len.max(5)])
    }

    pub fn expect_register<'b>(&'b mut self) -> ReadResult<'a, Register> {
        parse_alternatives!(
            self,
            b"B",
            Register::B(()),
            b"C",
            Register::C(()),
            b"D",
            Register::D(()),
            b"E",
            Register::E(()),
            b"H",
            Register::H(()),
            b"L",
            Register::L(()),
            b"M",
            Register::M(()),
            b"A",
            Register::A(()),
        )
    }

    pub fn expect_register_pair<'b>(&'b mut self) -> ReadResult<'a, RegisterPair> {
        parse_alternatives!(
            self,
            b"BC",
            RegisterPair::Bc(()),
            b"DE",
            RegisterPair::De(()),
            b"HL",
            RegisterPair::Hl(()),
            b"SP",
            RegisterPair::Sp(()),
        )
    }

    pub fn expect_hex<'b>(&'b mut self) -> ReadResult<'a, u8> {
        parse_alternatives!(
            self, b"0", 0x0, b"1", 0x1, b"2", 0x2, b"3", 0x3, b"4", 0x4, b"5", 0x5, b"6", 0x6,
            b"7", 0x7, b"8", 0x8, b"9", 0x9, b"a", 0xa, b"b", 0xb, b"c", 0xc, b"d", 0xd, b"e", 0xe,
            b"f", 0xf,
        )
    }

    pub fn expect_hex_8<'b>(&'b mut self) -> ReadResult<'a, u8> {
        todo!()
        // self.expect(b'0')?;
    }

    pub fn read_amount_bytes(&self) -> usize {
        self.original.len() - self.buffer.len()
    }
}
