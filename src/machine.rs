use crate::{
    coding::{self, reader::Reader},
    instruction::{Address, Condition, Data8, Data16, Instruction, Register, RegisterPair},
};

static MEMORY_SIZE_BYTES: usize = 2 << 16;
pub struct Memory([u8; MEMORY_SIZE_BYTES]);

impl Memory {
    pub fn new() -> Self {
        Self([0; MEMORY_SIZE_BYTES])
    }

    pub fn read_8(&self, address: Address) -> Data8 {
        self.0[address.value() as usize]
    }
    pub fn read_16(&self, address: Address) -> Option<Data16> {
        let low = self.0[address.value() as usize];
        let high = *self.0.get(address.value() as usize + 1)?;
        Some(Data16::new(low, high))
    }

    pub fn write_8(&mut self, address: Address, value: Data8) {
        self.0[address.value() as usize] = value;
    }
    pub fn write_16(&mut self, address: Address, value: Data16) -> Option<()> {
        self.0[address.value() as usize] = value.low;
        *self.0.get_mut(address.value() as usize + 1)? = value.high;

        Some(())
    }

    pub fn as_raw(&self) -> &[u8; MEMORY_SIZE_BYTES] {
        &self.0
    }
}

pub struct ConditionFlags {
    flags: [bool; 8],
}

impl ConditionFlags {
    pub fn new() -> Self {
        Self { flags: [false; 8] }
    }
    
    fn condition_index(condition: Condition) -> usize {
        match condition {
            Condition::Carry => 0,
            Condition::NoCarry => 1,
            Condition::Zero => 2,
            Condition::NoZero => 3,
            Condition::Positive => 4,
            Condition::Minus => 5,
            Condition::ParityEven => 6,
            Condition::ParityOdd => 7,
        }
    }
    
    pub fn get(&self, condition: Condition) -> bool {
        self.flags[Self::condition_index(condition)]
    }
    
    pub fn set(&mut self, condition: Condition, value: bool) {
        self.flags[Self::condition_index(condition)] = value;
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
            Register::B(..) => {
                return self.b;
            }
            Register::C(..) => {
                return self.c;
            }
            Register::D(..) => {
                return self.d;
            }
            Register::E(..) => {
                return self.e;
            }
            Register::H(..) => {
                return self.h;
            }
            Register::L(..) => {
                return self.l;
            }
            Register::M(..) => {
                let address = self.get_16(RegisterPair::Hl(()));

                return memory.read_8(address);
            }
            Register::A(..) => {
                return self.a;
            }
        }
    }

    pub fn set_8(&mut self, register: Register, value: Data8, memory: &mut Memory) {
        match register {
            Register::B(..) => {
                self.b = value;
            }
            Register::C(..) => {
                self.c = value;
            }
            Register::D(..) => {
                self.d = value;
            }
            Register::E(..) => {
                self.e = value;
            }
            Register::H(..) => {
                self.h = value;
            }
            Register::L(..) => {
                self.l = value;
            }
            Register::M(..) => {
                let address = self.get_16(RegisterPair::Hl(()));

                memory.write_8(address, value);
            }
            Register::A(..) => {
                self.a = value;
            }
        }
    }

    pub fn get_16(&self, register: RegisterPair) -> Data16 {
        match register {
            RegisterPair::Bc(..) => Data16::new(self.c, self.b),
            RegisterPair::De(..) => Data16::new(self.e, self.d),
            RegisterPair::Hl(..) => Data16::new(self.l, self.h),
            RegisterPair::Sp(..) => self.sp,
        }
    }
    pub fn set_16(&mut self, register: RegisterPair, value: Data16) {
        match register {
            RegisterPair::Bc(..) => {
                self.c = value.low;
                self.b = value.high;
            }
            RegisterPair::De(..) => {
                self.e = value.low;
                self.d = value.high;
            }
            RegisterPair::Hl(..) => {
                self.l = value.low;
                self.h = value.high;
            }
            RegisterPair::Sp(..) => self.sp = value,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum MachineState {
    Running,
    Halted,
    InvalidInstruction,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ExecutionResult {
    Running,
    ControlTransfer,
    Halt,
}

pub struct Machine {
    state: MachineState,
    memory: Box<Memory>,
    registers: RegisterMap,
    conditions: ConditionFlags,
    pc: Data16,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            state: MachineState::Running,
            memory: Box::new(Memory::new()),
            registers: RegisterMap::new(),
            conditions: ConditionFlags::new(),
            pc: Data16::ZERO,
        }
    }

    pub fn state(&self) -> MachineState {
        self.state
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

    pub fn run_cycle(&mut self) {
        match self.state {
            MachineState::Halted => {}
            MachineState::InvalidInstruction => {}
            MachineState::Running => {
                self.state = self.load_execute();
            }
        }
    }

    fn load_execute(&mut self) -> MachineState {
        let mut stream = Reader::new(&self.memory().0[self.pc().value() as usize..]);

        let Some(instruction) = coding::decode(&mut stream) else {
            return MachineState::InvalidInstruction;
        };
        let instruction_len = stream.read_amount_bytes();

        match self.execute(instruction) {
            ExecutionResult::Running => {
                self.pc = (self.pc.value() + instruction_len as u16).into();
                MachineState::Running
            }
            ExecutionResult::ControlTransfer => MachineState::Running,
            ExecutionResult::Halt => MachineState::Halted,
        }
    }

    fn execute(&mut self, instruction: Instruction) -> ExecutionResult {
        match instruction {
            Instruction::Mov(source, destination) => {
                self.registers.set_8(
                    destination,
                    self.registers.get_8(source, &self.memory),
                    &mut self.memory,
                );
                ExecutionResult::Running
            }
            Instruction::Mvi(destination, data) => {
                self.registers.set_8(destination, data, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Lxi(register_pair, data) => {
                self.registers.set_16(register_pair, data);
                ExecutionResult::Running
            }
            Instruction::Lda(_data) => unimplemented!(),
            Instruction::Sta(_data) => unimplemented!(),
            Instruction::Lhld(_data) => unimplemented!(),
            Instruction::Shld(_data) => unimplemented!(),
            Instruction::Ldax(_register_pair_indirect) => unimplemented!(),
            Instruction::Stax(_register_pair_indirect) => unimplemented!(),
            Instruction::Xchg => unimplemented!(),
            Instruction::Add(_register) => unimplemented!(),
            Instruction::Adi(_) => unimplemented!(),
            Instruction::Adc(_register) => unimplemented!(),
            Instruction::Aci(_) => unimplemented!(),
            Instruction::Sub(_register) => unimplemented!(),
            Instruction::Sui(_) => unimplemented!(),
            Instruction::Sbb(_register) => unimplemented!(),
            Instruction::Sbi(_) => unimplemented!(),
            Instruction::Inr(_register) => unimplemented!(),
            Instruction::Dcr(_register) => unimplemented!(),
            Instruction::Inx(_register_pair) => unimplemented!(),
            Instruction::Dcx(_register_pair) => unimplemented!(),
            Instruction::Dad(_register_pair) => unimplemented!(),
            Instruction::Daa => unimplemented!(),
            Instruction::Ana(_register) => unimplemented!(),
            Instruction::Ani(_) => unimplemented!(),
            Instruction::Xra(_register) => unimplemented!(),
            Instruction::Xri(_) => unimplemented!(),
            Instruction::Ora(_register) => unimplemented!(),
            Instruction::Ori(_) => unimplemented!(),
            Instruction::Cmp(_register) => unimplemented!(),
            Instruction::Cpi(_) => unimplemented!(),
            Instruction::Rlc => unimplemented!(),
            Instruction::Rrc => unimplemented!(),
            Instruction::Ral => unimplemented!(),
            Instruction::Rar => unimplemented!(),
            Instruction::Cma => unimplemented!(),
            Instruction::Cmc => unimplemented!(),
            Instruction::Stc => unimplemented!(),
            Instruction::Jmp(_data) => unimplemented!(),
            Instruction::Jcc(_condition, _data) => unimplemented!(),
            Instruction::Call(_data) => unimplemented!(),
            Instruction::Ccc(_condition, _data) => unimplemented!(),
            Instruction::Ret => unimplemented!(),
            Instruction::Rcc(_condition) => unimplemented!(),
            Instruction::Rst(_restart_number) => unimplemented!(),
            Instruction::Pchl => unimplemented!(),
            Instruction::Push(_register_pair_or_status) => unimplemented!(),
            Instruction::Pop(_register_pair_or_status) => unimplemented!(),
            Instruction::Xthl => unimplemented!(),
            Instruction::Sphl => unimplemented!(),
            Instruction::In(_) => unimplemented!(),
            Instruction::Out(_) => unimplemented!(),
            Instruction::Ei => unimplemented!(),
            Instruction::Di => unimplemented!(),
            Instruction::Hlt => ExecutionResult::Halt,
            Instruction::Nop => ExecutionResult::Running,
        }
    }
}
