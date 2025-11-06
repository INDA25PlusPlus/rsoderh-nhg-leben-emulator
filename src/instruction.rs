use std::{fmt::Display, ops::Add};

use parsable::Parsable;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Parsable)]
pub enum Register {
    A(#[literal = b"A"] ()) = 0b111,
    B(#[literal = b"A"] ()) = 0b000,
    C(#[literal = b"A"] ()) = 0b001,
    D(#[literal = b"A"] ()) = 0b010,
    E(#[literal = b"A"] ()) = 0b011,
    H(#[literal = b"A"] ()) = 0b100,
    L(#[literal = b"A"] ()) = 0b101,
    M(#[literal = b"A"] ()) = 0b110,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Register::B(..) => "B",
            Register::C(..) => "C",
            Register::D(..) => "D",
            Register::E(..) => "E",
            Register::H(..) => "H",
            Register::L(..) => "L",
            Register::M(..) => "M",
            Register::A(..) => "A",
        })
    }
}

impl TryFrom<u8> for Register {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b111 => Ok(Self::A(())),
            0b000 => Ok(Self::B(())),
            0b001 => Ok(Self::C(())),
            0b010 => Ok(Self::D(())),
            0b011 => Ok(Self::E(())),
            0b100 => Ok(Self::H(())),
            0b101 => Ok(Self::L(())),
            0b110 => Ok(Self::M(())),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Parsable)]
pub enum RegisterPair {
    Bc(#[literal = b"B"] ()) = 0b00,
    De(#[literal = b"B"] ()) = 0b01,
    Hl(#[literal = b"B"] ()) = 0b10,
    Sp(#[literal = b"B"] ()) = 0b11,
}

impl Display for RegisterPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RegisterPair::Bc(..) => "BC",
            RegisterPair::De(..) => "DE",
            RegisterPair::Hl(..) => "HL",
            RegisterPair::Sp(..) => "SP",
        })
    }
}

impl TryFrom<u8> for RegisterPair {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::Bc(())),
            0b01 => Ok(Self::De(())),
            0b10 => Ok(Self::Hl(())),
            0b11 => Ok(Self::Sp(())),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RegisterPairIndirect {
    Bc = 0b00,
    De = 0b01,
}

impl TryFrom<u8> for RegisterPairIndirect {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::Bc),
            0b01 => Ok(Self::De),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RegisterPairOrStatus {
    Bc = 0b00,
    De = 0b01,
    Hl = 0b10,
    StatusWord = 0b11,
}

impl TryFrom<u8> for RegisterPairOrStatus {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Self::Bc),
            0b01 => Ok(Self::De),
            0b10 => Ok(Self::Hl),
            0b11 => Ok(Self::StatusWord),
            _ => Err(()),
        }
    }
}

pub type Data8 = u8;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Data16 {
    pub low: Data8,
    pub high: Data8,
}

impl Data16 {
    pub const ZERO: Self = Data16 { low: 0, high: 0 };

    pub fn new(low: Data8, high: Data8) -> Self {
        Self { low, high }
    }

    pub fn value(&self) -> u16 {
        self.low as u16 + ((self.high as u16) << 8)
    }
}

impl From<u16> for Data16 {
    fn from(value: u16) -> Self {
        Self {
            low: (value & 0b1111_1111) as u8,
            high: (value >> 8) as u8,
        }
    }
}

impl Add for Data16 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        (self.value() + rhs.value()).into()
    }
}

pub type Address = Data16;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Condition {
    Carry = 0b011,
    NoCarry = 0b10,
    Zero = 0b001,
    NoZero = 0b000,
    Positive = 0b110,
    Minus = 0b111,
    ParityEven = 0b101,
    ParityOdd = 0b100,
}

impl TryFrom<u8> for Condition {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b011 => Ok(Self::Carry),
            0b10 => Ok(Self::NoCarry),
            0b001 => Ok(Self::Zero),
            0b000 => Ok(Self::NoZero),
            0b110 => Ok(Self::Positive),
            0b111 => Ok(Self::Minus),
            0b101 => Ok(Self::ParityEven),
            0b100 => Ok(Self::ParityOdd),
            _ => Err(()),
        }
    }
}

pub type Port = Data8;

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RestartNumber {
    R0 = 0b000,
    R1 = 0b001,
    R2 = 0b010,
    R3 = 0b011,
    R4 = 0b100,
    R5 = 0b101,
    R6 = 0b110,
    R7 = 0b111,
}

impl TryFrom<u8> for RestartNumber {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b000 => Ok(Self::R0),
            0b001 => Ok(Self::R1),
            0b010 => Ok(Self::R2),
            0b011 => Ok(Self::R3),
            0b100 => Ok(Self::R4),
            0b101 => Ok(Self::R5),
            0b110 => Ok(Self::R6),
            0b111 => Ok(Self::R7),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Instruction {
    // Data Transfer Group
    /// Move register / Move from memory / Move to memory
    Mov(Register, Register),
    /// Move immediate / Move to memory immediate
    Mvi(Register, Data8),
    /// Load register pair immediate
    Lxi(RegisterPair, Data16),
    /// Load accumulator direct
    Lda(Address),
    /// Store accumulator direct
    Sta(Address),
    /// Load H and L direct
    Lhld(Address),
    /// Store H and L direct
    Shld(Address),
    /// Load accumulator indirect
    Ldax(RegisterPairIndirect),
    /// Store accumulator indirect
    Stax(RegisterPairIndirect),
    /// Exchange H and L with D and E
    Xchg,

    // Arithmetic Group
    /// Add register / Add memory
    Add(Register),
    /// Add immediate
    Adi(Data8),
    /// Add register with carry / Add memory with carry
    Adc(Register),
    /// Add immediate with carry
    Aci(Data8),
    /// Subtract register / Subtract memory
    Sub(Register),
    /// Subtract immediate
    Sui(Data8),
    /// Subtract register with borrow / Subtract memory with borrow
    Sbb(Register),
    /// Subtract immediate with borrow
    Sbi(Data8),
    /// Increment register / Increment memory
    Inr(Register),
    /// Decrement register / Increment memory
    Dcr(Register),
    /// Increment register pair
    Inx(RegisterPair),
    /// Decrement register pair
    Dcx(RegisterPair),
    /// Add register pair to H and L
    Dad(RegisterPair),
    /// Decimal adjust accumulator
    Daa,

    // Logical Group
    /// AND register / AND memory
    Ana(Register),
    /// AND immediate
    Ani(Data8),
    /// XOR register / XOR memory
    Xra(Register),
    /// XOR immediate
    Xri(Data8),
    /// OR register / OR memory
    Ora(Register),
    /// OR immediate
    Ori(Data8),
    /// Compare register / Compare memory
    Cmp(Register),
    /// Compare immediate
    Cpi(Data8),
    /// Rotate left
    Rlc,
    /// Rotate right
    Rrc,
    /// Rotate left through carry
    Ral,
    /// Rotate right through carry
    Rar,
    /// Complement accumulator
    Cma,
    /// Complement carry
    Cmc,
    /// Set carry
    Stc,

    // Branch Group
    /// Jump
    Jmp(Address),
    /// Conditional jump
    Jcc(Condition, Address),
    /// Call
    Call(Address),
    /// Conditional call
    Ccc(Condition, Address),
    /// Return
    Ret,
    /// Conditional return
    Rcc(Condition),
    /// Restart
    Rst(RestartNumber),
    /// Jump H and L indirect - move H and L to PC
    Pchl,

    // Stack, I/O, and Machine Control Group
    /// Push / Push processor status word
    Push(RegisterPairOrStatus),
    /// Pop / Pop processor status word
    Pop(RegisterPairOrStatus),
    /// Exchange stack top with H and L
    Xthl,
    /// Move HL to SP
    Sphl,
    /// Input
    In(Port),
    /// Output
    Out(Port),
    /// Enable interrupts
    Ei,
    /// Disable interrupts
    Di,
    /// Halt
    Hlt,
    /// No op
    Nop,
}
