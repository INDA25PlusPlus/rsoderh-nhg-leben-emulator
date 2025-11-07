use std::io;

use crate::instruction::{Address, Condition, Data8, Data16, Port, Register, RegisterPair, RegisterPairIndirect, RegisterPairOrStatus, RestartNumber};

fn bits_write_offset(value: u8, insert: u8, index: u8) -> u8 {
    value | (insert << index)
}

fn write_opcode(stream: &mut impl io::Write, opcode: u8) -> io::Result<()> {
    stream.write(&[opcode]).map(|_| ())
}

fn write_opcode_rp(stream: &mut impl io::Write, opcode: u8, rp: RegisterPair) -> io::Result<()> {
    let opcode = bits_write_offset(opcode, rp.repr(), 4);
    write_opcode(stream, opcode)
}

fn write_opcode_rp_indirect(stream: &mut impl io::Write, opcode: u8, rp: RegisterPairIndirect) -> io::Result<()> {
    let opcode = bits_write_offset(opcode, rp.repr(), 4);
    write_opcode(stream, opcode)
}

fn write_opcode_rp_or_status(stream: &mut impl io::Write, opcode: u8, rp: RegisterPairOrStatus) -> io::Result<()> {
    let opcode = bits_write_offset(opcode, rp.repr(), 4);
    write_opcode(stream, opcode)
}

fn write_opcode_ddd(stream: &mut impl io::Write, opcode: u8, ddd: Register) -> io::Result<()> {
    let opcode = bits_write_offset(opcode, ddd.repr(), 3);
    write_opcode(stream, opcode)
}

fn write_opcode_sss(stream: &mut impl io::Write, opcode: u8, sss: Register) -> io::Result<()> {
    let opcode = bits_write_offset(opcode, sss.repr(), 0);
    write_opcode(stream, opcode)
}

fn write_opcode_ddd_sss(stream: &mut impl io::Write, opcode: u8, ddd: Register, sss: Register) -> io::Result<()> {
    let opcode = bits_write_offset(opcode, ddd.repr(), 3);
    let opcode = bits_write_offset(opcode, sss.repr(), 0);
    write_opcode(stream, opcode)
}

fn write_opcode_cc(stream: &mut impl io::Write, opcode: u8, cc: Condition) -> io::Result<()> {
    let opcode = bits_write_offset(opcode, cc as u8, 3);
    write_opcode(stream, opcode)
}

fn write_opcode_restart_number(stream: &mut impl io::Write, opcode: u8, n: RestartNumber) -> io::Result<()> {
    let opcode = bits_write_offset(opcode, n as u8, 3);
    write_opcode(stream, opcode)
}

fn write_data_8(stream: &mut impl io::Write, data: Data8) -> io::Result<()> {
    stream.write(&[data]).map(|_| ())
}

fn write_data_16(stream: &mut impl io::Write, data: Data16) -> io::Result<()> {
    stream.write(&[data.low, data.high]).map(|_| ())
}

fn write_addr(stream: &mut impl io::Write, addr: Address) -> io::Result<()> {
    let data: Data16 = addr.into();
    stream.write(&[data.low, data.high]).map(|_| ())
}

fn write_port(stream: &mut impl io::Write, port: Port) -> io::Result<()> {
    stream.write(&[port]).map(|_| ())
}

pub fn encode_noop<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0000_0000)
}

pub fn encode_lxi<'a>(stream: &mut impl io::Write, rp: RegisterPair, data: Data16) -> io::Result<()> {
    write_opcode_rp(stream, 0b0000_0000, rp)?;
    write_data_16(stream, data)
}

pub fn encode_stax<'a>(stream: &mut impl io::Write, rp: RegisterPairIndirect) -> io::Result<()> {
    write_opcode_rp_indirect(stream, 0b0000_0010, rp)
}

pub fn encode_inx<'a>(stream: &mut impl io::Write, rp: RegisterPair) -> io::Result<()> {
    write_opcode_rp(stream, 0b0000_0011, rp)
}

pub fn encode_inr<'a>(stream: &mut impl io::Write, ddd: Register) -> io::Result<()> {
    write_opcode_ddd(stream, 0b0000_0100, ddd)
}

pub fn encode_dcr<'a>(stream: &mut impl io::Write, ddd: Register) -> io::Result<()> {
    write_opcode_ddd(stream, 0b0000_0101, ddd)
}

pub fn encode_mvi<'a>(stream: &mut impl io::Write, ddd: Register, data: Data8) -> io::Result<()> {
    write_opcode_ddd(stream, 0b0000_0110, ddd)?;
    write_data_8(stream, data)
}

pub fn encode_dad<'a>(stream: &mut impl io::Write, rp: RegisterPair) -> io::Result<()> {
    write_opcode_rp(stream, 0b0000_1001, rp)
}

pub fn encode_ldax<'a>(stream: &mut impl io::Write, rp: RegisterPairIndirect) -> io::Result<()> {
    write_opcode_rp_indirect(stream, 0b0000_1010, rp)
}

pub fn encode_dcx<'a>(stream: &mut impl io::Write, rp: RegisterPair) -> io::Result<()> {
    write_opcode_rp(stream, 0b0000_1011, rp)
}

pub fn encode_rlc<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0000_0111)
}

pub fn encode_rrc<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0000_1111)
}

pub fn encode_ral<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0001_0111)
}

pub fn encode_rar<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0001_1111)
}

pub fn encode_shld<'a>(stream: &mut impl io::Write, addr: Address) -> io::Result<()> {
    write_opcode(stream, 0b0010_0010)?;
    write_addr(stream, addr)
}

pub fn encode_daa<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0010_0111)
}

pub fn encode_lhld<'a>(stream: &mut impl io::Write, addr: Address) -> io::Result<()> {
    write_opcode(stream, 0b0010_1010)?;
    write_addr(stream, addr)
}

pub fn encode_cma<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0010_1010)
}

pub fn encode_sta<'a>(stream: &mut impl io::Write, addr: Address) -> io::Result<()> {
    write_opcode(stream, 0b0011_0010)?;
    write_addr(stream, addr)
}

pub fn encode_stc<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0011_0111)
}

pub fn encode_lda<'a>(stream: &mut impl io::Write, addr: Address) -> io::Result<()> {
    write_opcode(stream, 0b0011_1010)?;
    write_addr(stream, addr)
}

pub fn encode_cmc<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0011_1111)
}

pub fn encode_mov<'a>(stream: &mut impl io::Write, ddd: Register, sss: Register) -> io::Result<()> {
    write_opcode_ddd_sss(stream, 0b0100_0000, ddd, sss)
}

pub fn encode_hlt<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b0111_0110)
}

pub fn encode_add<'a>(stream: &mut impl io::Write, sss: Register) -> io::Result<()> {
    write_opcode_sss(stream, 0b1000_0000, sss)
}

pub fn encode_adc<'a>(stream: &mut impl io::Write, sss: Register) -> io::Result<()> {
    write_opcode_sss(stream, 0b1001_0000, sss)
}

pub fn encode_sub<'a>(stream: &mut impl io::Write, sss: Register) -> io::Result<()> {
    write_opcode_sss(stream, 0b1001_0000, sss)
}

pub fn encode_sbb<'a>(stream: &mut impl io::Write, sss: Register) -> io::Result<()> {
    write_opcode_sss(stream, 0b1001_1000, sss)
}

pub fn encode_ana<'a>(stream: &mut impl io::Write, sss: Register) -> io::Result<()> {
    write_opcode_sss(stream, 0b1010_0000, sss)
}

pub fn encode_xra<'a>(stream: &mut impl io::Write, sss: Register) -> io::Result<()> {
    write_opcode_sss(stream, 0b1010_1000, sss)
}

pub fn encode_ora<'a>(stream: &mut impl io::Write, sss: Register) -> io::Result<()> {
    write_opcode_sss(stream, 0b1011_0000, sss)
}

pub fn encode_cmp<'a>(stream: &mut impl io::Write, sss: Register) -> io::Result<()> {
    write_opcode_sss(stream, 0b1011_1000, sss)
}

pub fn encode_rcc<'a>(stream: &mut impl io::Write, cc: Condition) -> io::Result<()> {
    write_opcode_cc(stream, 0b1100_0000, cc)
}

pub fn encode_pop<'a>(stream: &mut impl io::Write, rp: RegisterPairOrStatus) -> io::Result<()> {
    write_opcode_rp_or_status(stream, 0b1100_0001, rp)
}

pub fn encode_jcc<'a>(stream: &mut impl io::Write, cc: Condition, addr: Address) -> io::Result<()> {
    write_opcode_cc(stream, 0b1100_0010, cc)?;
    write_addr(stream, addr)
}

pub fn encode_jmp<'a>(stream: &mut impl io::Write, addr: Address) -> io::Result<()> {
    write_opcode(stream, 0b1100_0011)?;
    write_addr(stream, addr)
}

pub fn encode_ccc<'a>(stream: &mut impl io::Write, cc: Condition, addr: Address) -> io::Result<()> {
    write_opcode_cc(stream, 0b1100_0100, cc)?;
    write_addr(stream, addr)
}

pub fn encode_push<'a>(stream: &mut impl io::Write, rp: RegisterPairOrStatus) -> io::Result<()> {
    write_opcode_rp_or_status(stream, 0b1100_0101, rp)
}

// ADI ACI SUI SBI ANI XRI ORI CPI... (I skipped these)

pub fn encode_rst<'a>(stream: &mut impl io::Write, n: RestartNumber) -> io::Result<()> {
    write_opcode_restart_number(stream, 0b1100_0111, n)
}

pub fn encode_ret<'a>(stream: &mut impl io::Write) -> io::Result<()> {
    write_opcode(stream, 0b1100_1001)
}

pub fn encode_call<'a>(stream: &mut impl io::Write, addr: Address) -> io::Result<()> {
    write_opcode(stream, 0b1100_1101)?;
    write_addr(stream, addr)
}

pub fn encode_out<'a>(stream: &mut impl io::Write, port: Port) -> io::Result<()> {
    write_opcode(stream, 0b1101_0011)?;
    write_port(stream, port)
}

pub fn encode_in<'a>(stream: &mut impl io::Write, port: Port) -> io::Result<()> {
    write_opcode(stream, 0b1101_1011)?;
    write_port(stream, port)
}

