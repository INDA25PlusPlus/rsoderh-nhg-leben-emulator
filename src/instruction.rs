use std::{fmt::Display, ops::Add};

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Register {
    A = 0b111,
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    M = 0b110,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Register::B => "B",
            Register::C => "C",
            Register::D => "D",
            Register::E => "E",
            Register::H => "H",
            Register::L => "L",
            Register::M => "M",
            Register::A => "A",
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RegisterPair {
    Bc = 0b00,
    De = 0b01,
    Hl = 0b10,
    Sp = 0b11,
}

impl Display for RegisterPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RegisterPair::Bc => "BC",
            RegisterPair::De => "DE",
            RegisterPair::Hl => "HL",
            RegisterPair::Sp => "SP",
        })
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RegisterPairIndirect {
    Bc = 0b00,
    De = 0b01,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum RegisterPairOrStatus {
    Bc = 0b00,
    De = 0b01,
    Hl = 0b10,
    StatusWord = 0b11,
}

pub type Data8 = u8;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Data16 {
    low: Data8,
    high: Data8,
}

impl Data16 {
    pub const ZERO: Self = Data16 { low: 0, high: 0 };

    pub fn new(low: Data8, high: Data8) -> Self {
        Self { low, high }
    }

    pub fn value(&self) -> u16 {
        self.low as u16 + (self.high as u16) << 8
    }
}

impl From<u16> for Data16 {
    fn from(value: u16) -> Self {
        Self {
            low: value as u8,
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Instruction {

    // Data Transfer Group

    Mov(Register, Register),    // Move register / Move from memory / Move to memory
    Mvi(Register, Data8),       // Move immediate / Move to memory immediate
    Lxi(RegisterPair, Data16),  // Load register pair immediate
    Lda(Address),               // Load accumulator direct
    Sta(Address),               // Store accumulator direct
    Lhld(Address),              // Load H and L direct
    Shld(Address),              // Store H and L direct
    Ldax(RegisterPairIndirect), // Load accumulator indirect
    Stax(RegisterPairIndirect), // Store accumulator indirect
    Xchg,                       // Exchange H and L with D and E

    // Arithmetic Group

    Add(Register),              // Add register / Add memory
    Adi(Data8),                 // Add immediate
    Adc(Register),              // Add register with carry / Add memory with carry
    Aci(Data8),                 // Add immediate with carry
    Sub(Register),              // Subtract register / Subtract memory
    Sui(Data8),                 // Subtract immediate
    Sbb(Register),              // Subtract register with borrow / Subtract memory with borrow
    Sbi(Data8),                 // Subtract immediate with borrow
    Inr(Register),              // Increment register / Increment memory
    Dcr(Register),              // Decrement register / Increment memory
    Inx(RegisterPair),          // Increment register pair
    Dcx(RegisterPair),          // Decrement register pair
    Dad(RegisterPair),          // Add register pair to H and L
    Daa,                        // Decimal adjust accumulator

    // Logical Group

    Ana(Register),              // AND register / AND memory
    Ani(Data8),                 // AND immediate
    Xra(Register),              // XOR register / XOR memory
    Xri(Data8),                 // XOR immediate
    Ora(Register),              // OR register / OR memory
    Ori(Data8),                 // OR immediate
    Cmp(Register),              // Compare register / Compare memory
    Cpi(Data8),                 // Compare immediate
    Rlc,                        // Rotate left
    Rrc,                        // Rotate right
    Ral,                        // Rotate left through carry
    Rar,                        // Rotate right through carry
    Cma,                        // Complement accumulator
    Cmc,                        // Complement carry
    Stc,                        // Set carry

    // Branch Group

    Jmp(Address),               // Jump
    Jcc(Condition, Address),    // Conditional jump
    Call(Address),              // Call
    Ccc(Condition, Address),    // Conditional call
    Ret,                        // Return
    Rcc(Condition),             // Conditional return
    Rst(RestartNumber),         // Restart
    Pchl,                       // Jump H and L indirect - move H and L to PC

    // Stack, I/O, and Machine Control Group

    Push(RegisterPairOrStatus), // Push / Push processor status word
    Pop(RegisterPairOrStatus),  // Pop / Pop processor status word
    Xthl,                       // Exchange stack top with H and L
    Sphl,                       // Move HL to SP
    In(Port),                   // Input
    Out(Port),                  // Output
    Ei,                         // Enable interrupts
    Di,                         // Disable interrupts
    Hlt,                        // Halt
    Nop,                        // No op
}
