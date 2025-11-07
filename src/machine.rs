use std::io::{self, Read};

use crate::{
    coding::{self, reader::Reader},
    instruction::{
        Address, Condition, Data8, Data16, Instruction, Register, RegisterPair,
        RegisterPairOrStatus,
    },
};

static MEMORY_SIZE_BYTES: usize = 2 << 16;
pub struct Memory([u8; MEMORY_SIZE_BYTES]);

impl Memory {
    pub fn new() -> Self {
        Self([0; MEMORY_SIZE_BYTES])
    }

    pub fn read_8(&self, address: Address) -> Data8 {
        self.0[address as usize]
    }
    pub fn read_16(&self, address: Address) -> Option<Data16> {
        let low = self.0[address as usize];
        let high = *self.0.get(address as usize + 1)?;
        Some(Data16::new(low, high))
    }

    pub fn write_8(&mut self, address: Address, value: Data8) {
        self.0[address as usize] = value;
    }
    #[must_use]
    pub fn write_16(&mut self, address: Address, value: Data16) -> Option<()> {
        self.0[address as usize] = value.low;
        *self.0.get_mut(address as usize + 1)? = value.high;

        Some(())
    }

    pub fn write_slice(&mut self, address: Address, value: &[u8]) -> Option<()> {
        let range = (address as usize)..((address as usize) + value.len());
        self.0
            .get_mut(range)
            .map(|dest| dest.copy_from_slice(value))
    }

    pub fn as_raw(&self) -> &[u8; MEMORY_SIZE_BYTES] {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ConditionRegister {
    Carry,
    AuxiliaryCarry,
    Sign,
    Zero,
    Parity,
}

pub struct ConditionRegisters {
    flags: [bool; 5],
}

impl ConditionRegisters {
    pub fn new() -> Self {
        Self { flags: [false; 5] }
    }
    fn condition_index(condition: ConditionRegister) -> usize {
        match condition {
            ConditionRegister::Carry => 0,
            ConditionRegister::AuxiliaryCarry => 1,
            ConditionRegister::Zero => 2,
            ConditionRegister::Sign => 3,
            ConditionRegister::Parity => 4,
        }
    }

    pub fn get(&self, condition: ConditionRegister) -> bool {
        self.flags[Self::condition_index(condition)]
    }

    pub fn set(&mut self, condition: ConditionRegister, value: bool) {
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

                return memory.read_8(address.into());
            }
            Register::A => {
                return self.a;
            }
        }
    }

    pub fn set_8(&mut self, register: Register, value: Data8, memory: &mut Memory) {
        match register {
            Register::B => {
                self.b = value;
            }
            Register::C => {
                self.c = value;
            }
            Register::D => {
                self.d = value;
            }
            Register::E => {
                self.e = value;
            }
            Register::H => {
                self.h = value;
            }
            Register::L => {
                self.l = value;
            }
            Register::M => {
                let address = self.get_16(RegisterPair::Hl);

                memory.write_8(address.into(), value);
            }
            Register::A => {
                self.a = value;
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
    pub fn set_16(&mut self, register: RegisterPair, value: Data16) {
        match register {
            RegisterPair::Bc => {
                self.c = value.low;
                self.b = value.high;
            }
            RegisterPair::De => {
                self.e = value.low;
                self.d = value.high;
            }
            RegisterPair::Hl => {
                self.l = value.low;
                self.h = value.high;
            }
            RegisterPair::Sp => self.sp = value,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum HaltReason {
    HaltInstruction,
    InvalidInstruction,
    StackOverflow,
    StackUnderflow,
    MemoryOverflow,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum MachineState {
    Running,
    Halted(HaltReason),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ExecutionResult {
    Running,
    ControlTransfer,
    Halt,
    StackOverflow,
    // Is generated when the stack is popped too many times.
    StackUnderflow,
    // When an instruction attempts to write a 16-bit value to the very last byte of memory
    MemoryOverflow,
}

pub struct Machine {
    state: MachineState,
    memory: Box<Memory>,
    registers: RegisterMap,
    conditions: ConditionRegisters,
    pc: Data16,
    pub stdout: Vec<u8>,
}

fn is_even(value: u32) -> bool {
    value % 2 == 0
}

impl Machine {
    pub fn new() -> Self {
        Self {
            state: MachineState::Running,
            memory: Box::new(Memory::new()),
            registers: RegisterMap::new(),
            conditions: ConditionRegisters::new(),
            pc: Data16::ZERO,
            stdout: Vec::new(),
        }
    }

    pub fn state(&self) -> MachineState {
        self.state
    }

    pub fn registers(&self) -> &RegisterMap {
        &self.registers
    }

    pub fn conditions(&self) -> &ConditionRegisters {
        &self.conditions
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.memory
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

    #[must_use]
    pub fn stack_push(&mut self, data: Data16) -> Option<()> {
        let new_sp = self.register_16(RegisterPair::Sp).checked_sub(2)?;

        self.memory.write_16(new_sp.value(), data)?;
        self.registers.set_16(RegisterPair::Sp, new_sp);

        Some(())
    }

    pub fn stack_pop(&mut self) -> Option<Data16> {
        let value = self
            .memory
            .read_16(self.register_16(RegisterPair::Sp).value())?;
        self.registers.set_16(
            RegisterPair::Sp,
            self.register_16(RegisterPair::Sp).checked_add(2)?,
        );

        Some(value)
    }

    fn get_status_word(&self) -> Data16 {
        let cy_flag = self.conditions.get(ConditionRegister::Carry) as u8;
        let p_flag = self.conditions.get(ConditionRegister::Parity) as u8;
        let ac_flag = self.conditions.get(ConditionRegister::AuxiliaryCarry) as u8;
        let z_flag = self.conditions.get(ConditionRegister::Zero) as u8;
        let s_flag = self.conditions.get(ConditionRegister::Sign) as u8;
        let low = 0b0000_0000
            | cy_flag
            | (1 << 1)
            | (p_flag << 2)
            | (0 << 3)
            | (ac_flag << 4)
            | (0 << 5)
            | (z_flag << 6)
            | (s_flag << 7);
        let high = self.registers.get_8(Register::A, &self.memory);
        Data16 { low, high }
    }
    
    fn set_status_word(&mut self, data: Data16) {
        let Data16 { low, high } = data;

        let cy_flag = low & 0b0000_0001;
        let p_flag = (low >> 2) & 0b0000_0001;
        let ac_flag = (low >> 4) & 0b0000_0001;
        let z_flag = (low >> 6) & 0b0000_0001;
        let s_flag = (low >> 7) & 0b0000_0001;
        self.conditions.set(ConditionRegister::Carry, cy_flag == 1);
        self.conditions.set(ConditionRegister::Parity, p_flag == 1);
        self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag == 1);
        self.conditions.set(ConditionRegister::Zero, z_flag == 1);
        self.conditions.set(ConditionRegister::Sign, s_flag == 1);

        self.registers.set_8(Register::A, high, &mut self.memory);
    }

    pub fn run_cycle(&mut self) {
        match self.state {
            MachineState::Halted(_) => {}
            MachineState::Running => {
                self.state = self.load_execute();
            }
        }
    }

    fn load_execute(&mut self) -> MachineState {
        let mut stream = Reader::new(&self.memory().0[self.pc().value() as usize..]);

        let Some(instruction) = coding::decode(&mut stream) else {
            return MachineState::Halted(HaltReason::InvalidInstruction);
        };
        let instruction_len = stream.read_amount_bytes();

        let result = self.execute(instruction);
        if matches!(result, ExecutionResult::Running | ExecutionResult::Halt) {
            self.pc = (self.pc.value().wrapping_add(instruction_len as u16)).into();
        }

        match result {
            ExecutionResult::Running => MachineState::Running,
            ExecutionResult::ControlTransfer => MachineState::Running,
            ExecutionResult::Halt => MachineState::Halted(HaltReason::HaltInstruction),
            ExecutionResult::StackOverflow => MachineState::Halted(HaltReason::StackOverflow),
            ExecutionResult::StackUnderflow => MachineState::Halted(HaltReason::StackUnderflow),
            ExecutionResult::MemoryOverflow => MachineState::Halted(HaltReason::MemoryOverflow),
        }
    }
    
    pub fn load(&self) -> Option<Instruction> {
        let mut stream = Reader::new(&self.memory().0[self.pc().value() as usize..]);
        coding::decode(&mut stream)
    }

    fn execute(&mut self, instruction: Instruction) -> ExecutionResult {
        match instruction {
            Instruction::Mov(destination, source) => {
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
            Instruction::Lda(address) => {
                let mem = self.memory.read_8(address);
                self.registers.set_8(Register::A, mem, &mut self.memory);
                ExecutionResult::Running
            },
            Instruction::Sta(address) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                self.memory.write_8(address, a);
                ExecutionResult::Running
            },
            Instruction::Lhld(address) => {
                let Some(mem) = self.memory.read_16(address) else {
                    return ExecutionResult::MemoryOverflow;
                };
                self.registers.set_16(RegisterPair::Hl, mem);
                ExecutionResult::Running
            },
            Instruction::Shld(address) => {
                let hl = self.registers.get_16(RegisterPair::Hl);
                let res = self.memory.write_16(address, hl);
                if matches!(res, None) { return ExecutionResult::MemoryOverflow }
                ExecutionResult::Running
            },
            Instruction::Ldax(register_pair_indirect) => {
                let address = self.registers.get_16(register_pair_indirect.to_register_pair());
                let mem = self.memory.read_8(address.into());
                self.registers.set_8(Register::A, mem, &mut self.memory);
                ExecutionResult::Running
            },
            Instruction::Stax(register_pair_indirect) => {
                let address = self.registers.get_16(register_pair_indirect.to_register_pair());
                let a = self.registers.get_8(Register::A, &self.memory);
                self.memory.write_8(address.into(), a);
                ExecutionResult::Running
            },
            Instruction::Xchg => {
                let hl = self.registers.get_16(RegisterPair::Hl);
                let de = self.registers.get_16(RegisterPair::De);
                self.registers.set_16(RegisterPair::De, hl);
                self.registers.set_16(RegisterPair::Hl, de);
                ExecutionResult::Running
            },
            Instruction::Add(register) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let term = self.registers.get_8(register, &self.memory);
                
                let result = (a as u16) + (term as u16);

                let ac_flag = calc_ac_flag_add(a, term, false);
                let cy_flag = (result >> 8) & 0b1 == 1;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Adi(term) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                
                let result = (a as u16) + (term as u16);

                let ac_flag = calc_ac_flag_add(a, term, false);
                let cy_flag = (result >> 8) & 0b1 == 1;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Adc(register) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let term = self.registers.get_8(register, &self.memory);
                
                let cy_flag = self.conditions.get(ConditionRegister::Carry);
                let result = (a as u16) + (term as u16) + (cy_flag as u16);

                let ac_flag = calc_ac_flag_add(a, term, cy_flag);
                let cy_flag = (result >> 8) & 0b1 == 1;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Aci(term) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                
                let cy_flag = self.conditions.get(ConditionRegister::Carry);
                let result = (a as u16) + (term as u16) + (cy_flag as u16);

                let ac_flag = calc_ac_flag_add(a, term, cy_flag);
                let cy_flag = (result >> 8) & 0b1 == 1;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Sub(register) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let term = self.registers.get_8(register, &self.memory);

                let term_complement = (!term).wrapping_add(1);
                
                let result = (a as u16) + (term_complement as u16);

                let ac_flag = calc_ac_flag_add(a, term_complement, false);
                let cy_flag = (result >> 8) & 0b1 == 1;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Sui(term) => {
                let a = self.registers.get_8(Register::A, &self.memory);

                let term_complement = (!term).wrapping_add(1);
                
                let result = (a as u16) + (term_complement as u16);

                let ac_flag = calc_ac_flag_add(a, term_complement, false);
                let cy_flag = (result >> 8) & 0b1 == 1;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Sbb(register) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let term = self.registers.get_8(register, &self.memory);

                let cy_flag = self.conditions.get(ConditionRegister::Carry);
                let (term, borrow) = term.overflowing_add(cy_flag as u8);
                let term_complement = (!term).wrapping_add(1);
                
                let result = (a as u16) + (term_complement as u16);

                let ac_flag = calc_ac_flag_add(a, term_complement, false);
                let cy_flag = (result >> 8) & 0b1 == 1 || borrow;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Sbi(term) => {
                let a = self.registers.get_8(Register::A, &self.memory);

                let cy_flag = self.conditions.get(ConditionRegister::Carry);
                let (term, borrow) = term.overflowing_add(cy_flag as u8);
                let term_complement = (!term).wrapping_add(1);
                
                let result = (a as u16) + (term_complement as u16);

                let ac_flag = calc_ac_flag_add(a, term_complement, false);
                let cy_flag = (result >> 8) & 0b1 == 1 || borrow;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Inr(register) => {
                let value = self.registers.get_8(register, &self.memory);
                
                let result = value.wrapping_add(1);
                let ac_flag = calc_ac_flag_add(value, 1, false);
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());
                
                self.registers.set_8(register, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Dcr(register) => {
                let value = self.registers.get_8(register, &self.memory);
                
                let result = value.wrapping_sub(1);
                let ac_flag = calc_ac_flag_add(value, 0b1111_1111, false);
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());
                
                self.registers.set_8(register, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Inx(register_pair) => {
                let value: u16 = self.registers.get_16(register_pair).into();
                
                let result = value.wrapping_add(1);
                
                self.registers.set_16(register_pair, result.into());
                ExecutionResult::Running
            }
            Instruction::Dcx(register_pair) => {
                let value: u16 = self.registers.get_16(register_pair).into();
                
                let result = value.wrapping_sub(1);
                
                self.registers.set_16(register_pair, result.into());
                ExecutionResult::Running
            }
            Instruction::Dad(register_pair) => {
                let hl = self.registers.get_16(RegisterPair::Hl).value();
                let term = self.registers.get_16(register_pair).value();
                
                let result = (hl as u32) + (term as u32);
                let cy_flag = (result >> 16) & 0b1 == 1;

                self.registers.set_16(RegisterPair::Hl, (result as u16).into());
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                ExecutionResult::Running
            }
            Instruction::Daa => {
                // (Decimal Adjust Accumulator)
                //
                // The eight-bit number in the accumulator is adjusted
                // to form two four-bit Binary-Coded-Decimal digits by
                // the following process:
                //
                // 1. If the value of the least significant 4 bits of the
                //    accumulator is greater than 9 or if the AC flag
                //    is set, 6 is added to the accumulator.
                //
                // 2. If the value of the most significant 4 bits of the
                //    accumulator is now greater than 9, or if the CY
                //    flag is set, 6 is added to the most significant 4
                //    bits of the accumulator
                //
                let mut ac_flag = self.conditions.get(ConditionRegister::AuxiliaryCarry);
                let cy_flag = self.conditions.get(ConditionRegister::Carry);
                let mut wrapped = false;
                let mut a = self.registers.get_8(Register::A, &self.memory);
                // 1.
                let lsb = a & 0b0000_1111;
                if lsb > 9 || ac_flag {
                    ac_flag = lsb > 9;
                    if a > 0b1111_1111 - 6 {
                        wrapped = true;
                    }
                    a = a.wrapping_add(6);
                } else {
                    ac_flag = false;
                }
                // 2.
                let msb = (a >> 4) & 0b0000_1111;
                if msb > 9 || cy_flag {
                    if a > 0b1111_1111 - (6 << 4) {
                        wrapped = true;
                    }
                    a = a.wrapping_add(6 << 4);
                }

                let z_flag = a == 0;
                let s_flag = a & 0b1000_0000 == 1;
                let p_flag = is_even(a.count_ones());
                let cy_flag = wrapped;

                self.registers.set_8(Register::A, a, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            },
            Instruction::Ana(register) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let value = self.registers.get_8(register, &self.memory);
                
                let result = a & value;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, false);
                ExecutionResult::Running
            }
            Instruction::Ani(value) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                
                let result = a & value;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, false);
                ExecutionResult::Running
            }
            Instruction::Xra(register) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let value = self.registers.get_8(register, &self.memory);
                
                let result = a ^ value;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, false);
                ExecutionResult::Running
            }
            Instruction::Xri(value) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                
                let result = a ^ value;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, false);
                ExecutionResult::Running
            }
            Instruction::Ora(register) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let value = self.registers.get_8(register, &self.memory);
                
                let result = a | value;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, false);
                ExecutionResult::Running
            }
            Instruction::Ori(value) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                
                let result = a | value;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                self.registers.set_8(Register::A, result, &mut self.memory);
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, false);
                ExecutionResult::Running
            }
            Instruction::Cmp(register) => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let term = self.registers.get_8(register, &self.memory);

                let term_complement = (!term).wrapping_add(1);
                
                let result = (a as u16) + (term_complement as u16);

                let ac_flag = calc_ac_flag_add(a, term_complement, false);
                let cy_flag = (result >> 8) & 0b1 == 1;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                // subtraction without actually storing the value
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Cpi(term) => {
                let a = self.registers.get_8(Register::A, &self.memory);

                let term_complement = (!term).wrapping_add(1);
                
                let result = (a as u16) + (term_complement as u16);

                let ac_flag = calc_ac_flag_add(a, term_complement, false);
                let cy_flag = (result >> 8) & 0b1 == 1;
                let result = result as u8;
                let z_flag = result == 0;
                let s_flag = result & 0b1000_0000 == 1;
                let p_flag = is_even(result.count_ones());

                // subtraction without actually storing the value
                self.conditions.set(ConditionRegister::Zero, z_flag);
                self.conditions.set(ConditionRegister::Sign, s_flag);
                self.conditions.set(ConditionRegister::Parity, p_flag);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                self.conditions.set(ConditionRegister::AuxiliaryCarry, ac_flag);
                ExecutionResult::Running
            }
            Instruction::Rlc => {
                let cy_flag = (self.registers.a >> 7) & 0b1 == 1;
                self.registers.a = self.registers.a.wrapping_shl(1);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                ExecutionResult::Running
            },
            Instruction::Rrc => {
                let cy_flag = self.registers.a & 0b1 == 1;
                self.registers.a = self.registers.a.wrapping_shr(1);
                self.conditions.set(ConditionRegister::Carry, cy_flag);
                ExecutionResult::Running
            },
            Instruction::Ral => {
                let cy_flag = self.conditions.get(ConditionRegister::Carry);
                let new_cy_flag = (self.registers.a >> 7) & 0b1 == 1;
                self.registers.a = self.registers.a.wrapping_shl(1);
                self.registers.a &= cy_flag as u8;
                self.conditions.set(ConditionRegister::Carry, new_cy_flag);
                ExecutionResult::Running
            },
            Instruction::Rar => {
                let cy_flag = self.conditions.get(ConditionRegister::Carry);
                let new_cy_flag = self.registers.a & 0b1 == 1;
                self.registers.a = self.registers.a.wrapping_shr(1);
                self.registers.a &= (cy_flag as u8) << 7;
                self.conditions.set(ConditionRegister::Carry, new_cy_flag);
                ExecutionResult::Running
            },
            Instruction::Cma => {
                let a = self.registers.get_8(Register::A, &self.memory);
                let result = !a;
                self.registers.set_8(Register::A, result, &mut self.memory);
                ExecutionResult::Running
            },
            Instruction::Cmc => {
                let cy_flag = self.conditions.get(ConditionRegister::Carry);
                let result = !cy_flag;
                self.conditions.set(ConditionRegister::Carry, result);
                ExecutionResult::Running
            },
            Instruction::Stc => {
                self.conditions.set(ConditionRegister::Carry, true);
                ExecutionResult::Running
            },
            Instruction::Jmp(address) => {
                self.pc = address.into();
                ExecutionResult::ControlTransfer
            }
            Instruction::Jcc(condition, address) => {
                let should_jump = match condition {
                    Condition::Carry => self.conditions.get(ConditionRegister::Carry),
                    Condition::NoCarry => !self.conditions.get(ConditionRegister::Carry),
                    Condition::Zero => self.conditions.get(ConditionRegister::Zero),
                    Condition::NoZero => !self.conditions.get(ConditionRegister::Zero),
                    Condition::Minus => self.conditions.get(ConditionRegister::Sign),
                    Condition::Positive => !self.conditions.get(ConditionRegister::Sign),
                    Condition::ParityEven => self.conditions.get(ConditionRegister::Parity),
                    Condition::ParityOdd => !self.conditions.get(ConditionRegister::Parity),
                };
                if should_jump {
                    self.pc = address.into();
                    ExecutionResult::ControlTransfer
                } else {
                    ExecutionResult::Running
                }
            }
            Instruction::Call(address) => {
                if self.stack_push(self.pc).is_some() {
                    self.pc = address.into();
                    ExecutionResult::ControlTransfer
                } else {
                    ExecutionResult::StackOverflow
                }
            }
            Instruction::Ccc(condition, address) => {
                let should_call = match condition {
                    Condition::Carry => self.conditions.get(ConditionRegister::Carry),
                    Condition::NoCarry => !self.conditions.get(ConditionRegister::Carry),
                    Condition::Zero => self.conditions.get(ConditionRegister::Zero),
                    Condition::NoZero => !self.conditions.get(ConditionRegister::Zero),
                    Condition::Minus => self.conditions.get(ConditionRegister::Sign),
                    Condition::Positive => !self.conditions.get(ConditionRegister::Sign),
                    Condition::ParityEven => self.conditions.get(ConditionRegister::Parity),
                    Condition::ParityOdd => !self.conditions.get(ConditionRegister::Parity),
                };
                if should_call {
                    if should_call && self.stack_push(self.pc).is_some() {
                        self.pc = address.into();
                        ExecutionResult::ControlTransfer
                    } else {
                        ExecutionResult::StackOverflow
                    }
                } else {
                    ExecutionResult::Running
                }
            }
            Instruction::Ret => match self.stack_pop() {
                Some(address) => {
                    self.pc = address;
                    ExecutionResult::Running
                }
                None => ExecutionResult::StackUnderflow,
            },
            Instruction::Rcc(condition) => {
                let should_return = match condition {
                    Condition::Carry => self.conditions.get(ConditionRegister::Carry),
                    Condition::NoCarry => !self.conditions.get(ConditionRegister::Carry),
                    Condition::Zero => self.conditions.get(ConditionRegister::Zero),
                    Condition::NoZero => !self.conditions.get(ConditionRegister::Zero),
                    Condition::Minus => self.conditions.get(ConditionRegister::Sign),
                    Condition::Positive => !self.conditions.get(ConditionRegister::Sign),
                    Condition::ParityEven => self.conditions.get(ConditionRegister::Parity),
                    Condition::ParityOdd => !self.conditions.get(ConditionRegister::Parity),
                };

                if should_return {
                    match self.stack_pop() {
                        Some(address) => {
                            self.pc = address;
                            ExecutionResult::Running
                        }
                        None => ExecutionResult::StackUnderflow,
                    }
                } else {
                    ExecutionResult::Running
                }
            }
            Instruction::Rst(restart_number) => {
                if self.stack_push(self.pc).is_some() {
                    self.pc = (u16::from(restart_number) << 3).into();
                    ExecutionResult::ControlTransfer
                } else {
                    ExecutionResult::StackOverflow
                }
            },
            Instruction::Pchl => {
                self.pc = self.register_16(RegisterPair::Hl);
                ExecutionResult::ControlTransfer
            }
            Instruction::Push(register) => {
                let data = if let Some(register) = register.to_register_pair() {
                    self.register_16(register)
                } else {
                    self.get_status_word()
                };
                match self.stack_push(data) {
                    Some(()) => ExecutionResult::Running,
                    None => ExecutionResult::StackOverflow,
                }
            }
            Instruction::Pop(register) => {
                match self.stack_pop() {
                    Some(value) => {
                        if let Some(register) = register.to_register_pair() {
                            self.registers.set_16(register, value);
                        } else {
                            self.set_status_word(value)
                        }
                        ExecutionResult::Running
                    }
                    None => ExecutionResult::StackOverflow,
                }
            }
            Instruction::Xthl => {
                let hl = self.registers.get_16(RegisterPair::Hl);
                let sp = self.registers.get_16(RegisterPair::Sp);
                let Some(stack_top) = self.memory.read_16(sp.into()) else {
                    return ExecutionResult::StackOverflow;
                };
                self.registers.set_16(RegisterPair::Hl, stack_top);
                if matches!(self.memory.write_16(sp.into(), hl), None) {
                    return ExecutionResult::StackOverflow;
                }
                ExecutionResult::Running
            },
            Instruction::Sphl => {
                let hl = self.registers.get_16(RegisterPair::Hl);
                self.registers.set_16(RegisterPair::Sp, hl);
                ExecutionResult::Running
            },
            Instruction::In(port) => {
                let byte = match port {
                    0 => {
                        match io::stdin()
                            .bytes()
                            .next()
                            .map(|res| res.expect("surely io doesn't error"))
                        {
                            Some(byte) => byte,
                            None => return ExecutionResult::Halt,
                        }
                    }
                    _ => 0,
                };
                
                self.registers.set_8(Register::A, byte, &mut self.memory);

                ExecutionResult::Running
            }
            Instruction::Out(port) => {
                match port {
                    0 => {
                        let byte = self.register_8(Register::A);
                        self.stdout.push(byte);
                        ExecutionResult::Running
                    }
                    1 => {
                        let number = self.register_8(Register::A);
                        self.stdout.extend_from_slice(format!("{}", number).as_bytes());
                        ExecutionResult::Running
                    }
                    2 => {
                        let number = self.register_16(RegisterPair::Hl).value();
                        self.stdout.extend_from_slice(format!("{}", number).as_bytes());
                        ExecutionResult::Running
                    }
                    _ => ExecutionResult::Running,
                }
            },
            // We don't support interrupts, equate EI and DI to NOP
            Instruction::Ei => ExecutionResult::Running,
            Instruction::Di => ExecutionResult::Running,
            Instruction::Hlt => ExecutionResult::Halt,
            Instruction::Nop => ExecutionResult::Running,
        }
    }
}

fn calc_ac_flag_add(a: u8, b: u8, cy_flag: bool) -> bool {
    let a = a & 0b0000_1111;
    let b = b & 0b0000_1111;
    (a + b + (cy_flag as u8)) & 0b0001_0000 == 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_add_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        machine.conditions.set(ConditionRegister::Carry, true);

        machine
            .registers
            .set_8(Register::A, 0x80, &mut machine.memory);
        machine
            .registers
            .set_8(Register::B, 0x00, &mut machine.memory);

        let result = machine.execute(Instruction::Add(Register::B));
        let elapsed = now.elapsed();

        // assert_eq!(result, ExecutionResult::Running);

        // assert_eq!(0x6B, machine.register_8(Register::A));
        println!(
            "add: Time elapsed: {:?}, A: {:?}, Sign: {:?}",
            elapsed,
            machine.register_8(Register::A),
            machine.conditions.get(ConditionRegister::Sign)
        );
    }
    #[test]
    fn test_sub_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        // machine.conditions.set(ConditionRegister::Carry, true);

        machine
            .registers
            .set_8(Register::A, 0x20, &mut machine.memory);
        machine
            .registers
            .set_8(Register::B, 0x10, &mut machine.memory);
        let result = machine.execute(Instruction::Sbi(66));
        let elapsed = now.elapsed();

        // assert_eq!(result, ExecutionResult::Running);

        // assert_eq!(0x10, machine.register_8(Register::A));

        println!("sub: Time elapsed: {:?}", elapsed);
    }

    #[test]
    fn test_inr_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        machine
            .registers
            .set_8(Register::B, 0x00, &mut machine.memory);
        let result = machine.execute(Instruction::Inr(Register::B));
        let elapsed = now.elapsed();

        assert_eq!(result, ExecutionResult::Running);

        assert_eq!(0x01, machine.register_8(Register::B));

        println!("inr: Time elapsed: {:?}", elapsed);
    }

    #[test]
    fn test_inx_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        machine
            .registers
            .set_16(RegisterPair::Bc, 0xFF00.into());
        let result = machine.execute(Instruction::Inx(RegisterPair::Bc));
        let elapsed = now.elapsed();

        assert_eq!(result, ExecutionResult::Running);

        assert_eq!(0xFF01, machine.register_16(RegisterPair::Bc).value());

        println!("inx: Time elapsed: {:?}", elapsed);
    }
    #[test]
    fn test_ana_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        machine
            .registers
            .set_8(Register::A, 0xFC, &mut machine.memory);
        machine
            .registers
            .set_8(Register::B, 0x0F, &mut machine.memory);

        let result = machine.execute(Instruction::Ana(Register::B));

        let elapsed = now.elapsed();

        println!(
            "inx: Time elapsed: {:?}\nReturn: {:?}",
            elapsed,
            machine.registers.get_8(Register::A, &machine.memory)
        );
    }
}
