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
            ConditionRegister::AuxiliaryCarry => 0,
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

                return memory.read_8(address.into());
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

                memory.write_8(address.into(), value);
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
pub enum HaltReason {
    Instruction,
    InvalidInstruction,
    StackOverflow,
    StackUnderflow,
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
}

pub struct Machine {
    state: MachineState,
    memory: Box<Memory>,
    registers: RegisterMap,
    conditions: ConditionRegisters,
    pc: Data16,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            state: MachineState::Running,
            memory: Box::new(Memory::new()),
            registers: RegisterMap::new(),
            conditions: ConditionRegisters::new(),
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
        let new_sp = self.register_16(RegisterPair::Sp(())).checked_sub(2)?;

        self.memory.write_16(new_sp.value(), data)?;
        self.registers.set_16(RegisterPair::Sp(()), new_sp);

        Some(())
    }

    pub fn stack_pop(&mut self) -> Option<Data16> {
        let value = self
            .memory
            .read_16(self.register_16(RegisterPair::Sp(())).value())?;
        self.registers.set_16(
            RegisterPair::Sp(()),
            self.register_16(RegisterPair::Sp(())).checked_add(2)?,
        );

        Some(value)
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
            self.pc = (self.pc.value() + instruction_len as u16).into();
        }

        match result {
            ExecutionResult::Running => MachineState::Running,
            ExecutionResult::ControlTransfer => MachineState::Running,
            ExecutionResult::Halt => MachineState::Halted(HaltReason::Instruction),
            ExecutionResult::StackOverflow => MachineState::Halted(HaltReason::StackOverflow),
            ExecutionResult::StackUnderflow => MachineState::Halted(HaltReason::StackUnderflow),
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
            Instruction::Add(register) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let value = self.registers.get_8(register, &self.memory);
                let result = a.wrapping_add(value);
                if result < a.max(value) {
                    self.conditions.set(ConditionRegister::Carry, true);
                } else {
                    self.conditions.set(ConditionRegister::Carry, false);
                }
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Adi(data) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let result = a.wrapping_add(data);
                if result < a.max(data) {
                    self.conditions.set(ConditionRegister::Carry, true);
                }
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Adc(register) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let value = self.registers.get_8(register, &self.memory);
                let (mut result, overflow_1) = a.overflowing_add(value);
                let mut overflow_2 = false;
                if self.conditions.get(ConditionRegister::Carry) {
                    (result, overflow_2) = result.overflowing_add(1);
                }
                self.conditions
                    .set(ConditionRegister::Carry, overflow_1 || overflow_2);
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Aci(data) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let (mut result, overflow_1) = a.overflowing_add(data);
                let mut overflow_2 = false;
                if self.conditions.get(ConditionRegister::Carry) {
                    (result, overflow_2) = result.overflowing_add(1);
                }
                self.conditions
                    .set(ConditionRegister::Carry, overflow_1 || overflow_2);
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Sub(register) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let value = self.register_8(register);
                let result = a.wrapping_sub(value);
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Sui(data) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let result = a.wrapping_sub(data);
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Sbb(register) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let mut value = self.register_8(register);
                if self.conditions.get(ConditionRegister::Carry) {
                    value += 1;
                }
                let result = a.wrapping_sub(value);
                self.conditions.set(ConditionRegister::Carry, false);
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Sbi(data) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let mut result = a.wrapping_sub(data);
                if self.conditions.get(ConditionRegister::Carry) {
                    result = result.wrapping_add(1);
                }
                self.conditions.set(ConditionRegister::Carry, false);
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Inr(register) => {
                self.registers.set_8(
                    register,
                    self.registers.get_8(register, &self.memory).wrapping_add(1),
                    &mut self.memory,
                );
                ExecutionResult::Running
            }
            Instruction::Dcr(register) => {
                self.registers.set_8(
                    register,
                    self.registers.get_8(register, &self.memory).wrapping_sub(1),
                    &mut self.memory,
                );
                ExecutionResult::Running
            }
            Instruction::Inx(register_pair) => {
                self.registers.set_16(
                    register_pair,
                    ((self.registers.get_16(register_pair).high.wrapping_add(1) as u16) << 8
                        | self.registers.get_16(register_pair).low.wrapping_add(1) as u16)
                        .into(),
                );
                ExecutionResult::Running
            }
            Instruction::Dcx(register_pair) => {
                self.registers.set_16(
                    register_pair,
                    ((self.registers.get_16(register_pair).high.wrapping_sub(1) as u16) << 8
                        | self.registers.get_16(register_pair).low.wrapping_sub(1) as u16)
                        .into(),
                );
                ExecutionResult::Running
            }
            Instruction::Dad(register_pair) => {
                let hl = self.registers.get_16(RegisterPair::Hl(())).value();
                let value = self.registers.get_16(register_pair).value();
                let result = hl.wrapping_add(value);
                if result < hl.max(value) {
                    self.conditions.set(ConditionRegister::Carry, true);
                } else {
                    self.conditions.set(ConditionRegister::Carry, false);
                }
                self.registers.set_16(RegisterPair::Hl(()), result.into());
                ExecutionResult::Running
            }
            Instruction::Daa => unimplemented!(),
            Instruction::Ana(register) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let value = self.registers.get_8(register, &self.memory);
                let result = a & value;
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Ani(data) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let result = a & data;
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Xra(register) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let value = self.registers.get_8(register, &self.memory);
                let result = a ^ value;
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Xri(data) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let result = a ^ data;
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Ora(register) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let value = self.registers.get_8(register, &self.memory);
                let result = a | value;
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Ori(data) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let result = a | data;
                self.registers
                    .set_8(Register::A(()), result, &mut self.memory);
                ExecutionResult::Running
            }
            Instruction::Cmp(register) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let value = self.register_8(register);
                let (result, overflow) = a.overflowing_sub(value);
                self.conditions.set(ConditionRegister::Carry, !overflow);
                self.conditions.set(ConditionRegister::Zero, result == 0);
                ExecutionResult::Running
            }
            Instruction::Cpi(data) => {
                let a = self.registers.get_8(Register::A(()), &self.memory);
                let (result, overflow) = a.overflowing_sub(data);
                self.conditions.set(ConditionRegister::Carry, !overflow);
                self.conditions.set(ConditionRegister::Zero, result == 0);
                ExecutionResult::Running
            }
            Instruction::Rlc => unimplemented!(),
            Instruction::Rrc => unimplemented!(),
            Instruction::Ral => unimplemented!(),
            Instruction::Rar => unimplemented!(),
            Instruction::Cma => unimplemented!(),
            Instruction::Cmc => unimplemented!(),
            Instruction::Stc => unimplemented!(),
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
                if should_call && self.stack_push(self.pc).is_some() {
                    self.pc = address.into();
                    ExecutionResult::ControlTransfer
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
            Instruction::Rst(_restart_number) => unimplemented!(),
            Instruction::Pchl => {
                self.pc = self.register_16(RegisterPair::Hl(()));
                ExecutionResult::ControlTransfer
            }
            Instruction::Push(register_pair_or_status) => {
                let register = match register_pair_or_status {
                    RegisterPairOrStatus::Bc(..) => RegisterPair::Bc(()),
                    RegisterPairOrStatus::De(..) => RegisterPair::De(()),
                    RegisterPairOrStatus::Hl(..) => RegisterPair::Hl(()),
                    RegisterPairOrStatus::StatusWord(..) => unimplemented!(),
                };
                match self.stack_push(self.register_16(register)) {
                    Some(()) => ExecutionResult::Running,
                    None => ExecutionResult::StackOverflow,
                }
            }
            Instruction::Pop(register_pair_or_status) => {
                let register = match register_pair_or_status {
                    RegisterPairOrStatus::Bc(..) => RegisterPair::Bc(()),
                    RegisterPairOrStatus::De(..) => RegisterPair::De(()),
                    RegisterPairOrStatus::Hl(..) => RegisterPair::Hl(()),
                    RegisterPairOrStatus::StatusWord(..) => unimplemented!(),
                };
                match self.stack_pop() {
                    Some(value) => {
                        self.registers.set_16(register, value);
                        ExecutionResult::Running
                    }
                    None => ExecutionResult::StackOverflow,
                }
            }
            Instruction::Xthl => unimplemented!(),
            Instruction::Sphl => unimplemented!(),
            // We don't support io
            Instruction::In(_) => unimplemented!(),
            Instruction::Out(_) => unimplemented!(),
            // We don't support interrupts
            Instruction::Ei => unimplemented!(),
            Instruction::Di => unimplemented!(),
            Instruction::Hlt => ExecutionResult::Halt,
            Instruction::Nop => ExecutionResult::Running,
        }
    }
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
            .set_8(Register::A(()), 0x35, &mut machine.memory);
        machine
            .registers
            .set_8(Register::B(()), 0x35, &mut machine.memory);

        let result = machine.execute(Instruction::Adc(Register::B(())));
        let elapsed = now.elapsed();

        assert_eq!(result, ExecutionResult::Running);

        assert_eq!(0x6B, machine.register_8(Register::A(())));
        println!("add: Time elapsed: {:?}", elapsed);
    }
    #[test]
    fn test_sub_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        // machine.conditions.set(ConditionRegister::Carry, true);

        machine
            .registers
            .set_8(Register::A(()), 0x20, &mut machine.memory);
        machine
            .registers
            .set_8(Register::B(()), 0x10, &mut machine.memory);
        let result = machine.execute(Instruction::Sbi(66));
        let elapsed = now.elapsed();

        // assert_eq!(result, ExecutionResult::Running);

        // assert_eq!(0x10, machine.register_8(Register::A(())));

        println!("sub: Time elapsed: {:?}", elapsed);
    }

    #[test]
    fn test_inr_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        machine
            .registers
            .set_8(Register::B(()), 0x00, &mut machine.memory);
        let result = machine.execute(Instruction::Inr(Register::B(())));
        let elapsed = now.elapsed();

        assert_eq!(result, ExecutionResult::Running);

        assert_eq!(0x01, machine.register_8(Register::B(())));

        println!("inr: Time elapsed: {:?}", elapsed);
    }

    #[test]
    fn test_inx_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        machine
            .registers
            .set_16(RegisterPair::Bc(()), 0xFF00.into());
        let result = machine.execute(Instruction::Inx(RegisterPair::Bc(())));
        let elapsed = now.elapsed();

        assert_eq!(result, ExecutionResult::Running);

        assert_eq!(0x0001, machine.register_16(RegisterPair::Bc(())).value());

        println!("inx: Time elapsed: {:?}", elapsed);
    }
    #[test]
    fn test_ana_register() {
        let now = Instant::now();
        let mut machine = Machine::new();

        machine
            .registers
            .set_8(Register::A(()), 0xFC, &mut machine.memory);
        machine
            .registers
            .set_8(Register::B(()), 0x0F, &mut machine.memory);

        let result = machine.execute(Instruction::Ana(Register::B(())));

        let elapsed = now.elapsed();

        println!(
            "inx: Time elapsed: {:?}\nReturn: {:?}",
            elapsed,
            machine.registers.get_8(Register::A(()), &machine.memory)
        );
    }
}
