use crate::instruction::{Address, Data8, Data16, Register, RegisterPair};

static MEMORY_SIZE_BYTES: usize = 2 << 16;
pub struct Memory([u8; MEMORY_SIZE_BYTES]);

impl Memory {
    pub fn new() -> Self {
        Self([0; MEMORY_SIZE_BYTES])
    }

    pub fn read_u8(&self, address: Address) -> Data8 {
        self.0[address.value() as usize]
    }
    pub fn read_u16(&self, address: Address) -> Option<Data16> {
        let low = self.0[address.value() as usize];
        let high = *self.0.get(address.value() as usize + 1)?;
        Some(Data16::new(low, high))
    }

    pub fn as_raw(&self) -> &[u8; MEMORY_SIZE_BYTES] {
        &self.0
    }
}

// Struct containing program addressable registers.
pub struct RegisterMap {
    a: Data8,
    b: Data8,
    c: Data8,
    d: Data8,
    e: Data8,
    h: Data8,
    l: Data8,
    sp: Data16,
}

impl RegisterMap {
    pub fn new() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: Data16::ZERO,
        }
    }

    pub fn get_8(&self, register: Register, memory: &Memory) -> Data8 {
        match register {
            Register::B => {
                return self.b;
            }
            Register::C => {
                return self.c;
            }
            Register::D => {
                return self.d;
            }
            Register::E => {
                return self.e;
            }
            Register::H => {
                return self.h;
            }
            Register::L => {
                return self.l;
            }
            Register::M => {
                let address = self.get_16(RegisterPair::Hl);

                return memory.read_u8(address);
            }
            Register::A => {
                return self.a;
            }
        }
    }

    pub fn get_16(&self, register: RegisterPair) -> Data16 {
        match register {
            RegisterPair::Bc => Data16::new(self.c, self.b),
            RegisterPair::De => Data16::new(self.e, self.d),
            RegisterPair::Hl => Data16::new(self.l, self.h),
            RegisterPair::Sp => self.sp,
        }
    }
}

pub struct Machine {
    memory: Box<Memory>,
    registers: RegisterMap,
    pc: Data16,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            memory: Box::new(Memory::new()),
            registers: RegisterMap::new(),
            pc: Data16::ZERO,
        }
    }

    pub fn registers(&self) -> &RegisterMap {
        &self.registers
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn register_8(&self, register: Register) -> Data8 {
        self.registers().get_8(register, self.memory())
    }

    pub fn register_16(&self, register: RegisterPair) -> Data16 {
        self.registers().get_16(register)
    }

    pub fn pc(&self) -> Data16 {
        self.pc
    }
}
