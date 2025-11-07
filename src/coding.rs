use std::io::{self, Write};

use crate::{
    coding::reader::Reader,
    instruction::{Instruction, InstructionOrData},
};

mod decode;
mod encode;
pub mod reader;

pub fn encode_program(buffer: &mut impl Write, items: &[InstructionOrData]) -> io::Result<()> {
    for item in items {
        match item {
            InstructionOrData::Instruction(instruction) => {
                encode(buffer, *instruction)?;
            }
            InstructionOrData::Byte(data) => {
                buffer.write(&[*data])?;
            }
            InstructionOrData::Slice(slice) => {
                buffer.write_all(&slice)?;
            },
        }
    }
    
    Ok(())
}

pub fn encode(buffer: &mut impl Write, instruction: Instruction) -> std::io::Result<()> {
    match instruction {
        Instruction::Mov(register, register1) => encode::encode_mov(buffer, register, register1),
        Instruction::Mvi(register, data) => encode::encode_mvi(buffer, register, data),
        Instruction::Lxi(register_pair, data16) => {
            encode::encode_lxi(buffer, register_pair, data16)
        }
        Instruction::Lda(addr) => encode::encode_lda(buffer, addr),
        Instruction::Sta(addr) => encode::encode_sta(buffer, addr),
        Instruction::Lhld(addr) => encode::encode_lhld(buffer, addr),
        Instruction::Shld(addr) => encode::encode_shld(buffer, addr),
        Instruction::Ldax(register_pair_indirect) => {
            encode::encode_ldax(buffer, register_pair_indirect)
        }
        Instruction::Stax(register_pair_indirect) => {
            encode::encode_stax(buffer, register_pair_indirect)
        }
        Instruction::Xchg => encode::encode_xchg(buffer),
        Instruction::Add(register) => encode::encode_add(buffer, register),
        Instruction::Adi(data) => encode::encode_adi(buffer, data),
        Instruction::Adc(register) => encode::encode_adc(buffer, register),
        Instruction::Aci(data) => encode::encode_aci(buffer, data),
        Instruction::Sub(register) => encode::encode_sub(buffer, register),
        Instruction::Sui(data) => encode::encode_sui(buffer, data),
        Instruction::Sbb(register) => encode::encode_sbb(buffer, register),
        Instruction::Sbi(data) => encode::encode_sbi(buffer, data),
        Instruction::Inr(register) => encode::encode_inr(buffer, register),
        Instruction::Dcr(register) => encode::encode_dcr(buffer, register),
        Instruction::Inx(register_pair) => encode::encode_inx(buffer, register_pair),
        Instruction::Dcx(register_pair) => encode::encode_dcx(buffer, register_pair),
        Instruction::Dad(register_pair) => encode::encode_dad(buffer, register_pair),
        Instruction::Daa => encode::encode_daa(buffer),
        Instruction::Ana(register) => encode::encode_ana(buffer, register),
        Instruction::Ani(data) => encode::encode_ani(buffer, data),
        Instruction::Xra(register) => encode::encode_xra(buffer, register),
        Instruction::Xri(data) => encode::encode_xri(buffer, data),
        Instruction::Ora(register) => encode::encode_ora(buffer, register),
        Instruction::Ori(data) => encode::encode_ori(buffer, data),
        Instruction::Cmp(register) => encode::encode_cmp(buffer, register),
        Instruction::Cpi(data) => encode::encode_cpi(buffer, data),
        Instruction::Rlc => encode::encode_rlc(buffer),
        Instruction::Rrc => encode::encode_rrc(buffer),
        Instruction::Ral => encode::encode_ral(buffer),
        Instruction::Rar => encode::encode_rar(buffer),
        Instruction::Cma => encode::encode_cma(buffer),
        Instruction::Cmc => encode::encode_cmc(buffer),
        Instruction::Stc => encode::encode_stc(buffer),
        Instruction::Jmp(addr) => encode::encode_jmp(buffer, addr),
        Instruction::Jcc(condition, addr) => encode::encode_jcc(buffer, condition, addr),
        Instruction::Call(addr) => encode::encode_call(buffer, addr),
        Instruction::Ccc(condition, addr) => encode::encode_ccc(buffer, condition, addr),
        Instruction::Ret => encode::encode_ret(buffer),
        Instruction::Rcc(condition) => encode::encode_rcc(buffer, condition),
        Instruction::Rst(restart_number) => encode::encode_rst(buffer, restart_number),
        Instruction::Pchl => encode::encode_pchl(buffer),
        Instruction::Push(register_pair_or_status) => {
            encode::encode_push(buffer, register_pair_or_status)
        }
        Instruction::Pop(register_pair_or_status) => {
            encode::encode_pop(buffer, register_pair_or_status)
        }
        Instruction::Xthl => encode::encode_xthl(buffer),
        Instruction::Sphl => encode::encode_sphl(buffer),
        Instruction::In(port) => encode::encode_in(buffer, port),
        Instruction::Out(port) => encode::encode_out(buffer, port),
        Instruction::Ei => encode::encode_ei(buffer),
        Instruction::Di => encode::encode_di(buffer),
        Instruction::Hlt => encode::encode_hlt(buffer),
        Instruction::Nop => encode::encode_noop(buffer),
    }
}

pub fn decode<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    None.or_else(|| decode::parse_noop(stream))
        .or_else(|| decode::parse_lxi(stream))
        .or_else(|| decode::parse_stax(stream))
        .or_else(|| decode::parse_inx(stream))
        .or_else(|| decode::parse_inr(stream))
        .or_else(|| decode::parse_dcr(stream))
        .or_else(|| decode::parse_mvi(stream))
        .or_else(|| decode::parse_dad(stream))
        .or_else(|| decode::parse_ldax(stream))
        .or_else(|| decode::parse_dcx(stream))
        .or_else(|| decode::parse_rlc(stream))
        .or_else(|| decode::parse_rrc(stream))
        .or_else(|| decode::parse_ral(stream))
        .or_else(|| decode::parse_rar(stream))
        .or_else(|| decode::parse_shld(stream))
        .or_else(|| decode::parse_daa(stream))
        .or_else(|| decode::parse_lhld(stream))
        .or_else(|| decode::parse_cma(stream))
        .or_else(|| decode::parse_sta(stream))
        .or_else(|| decode::parse_stc(stream))
        .or_else(|| decode::parse_lda(stream))
        .or_else(|| decode::parse_cmc(stream))
        .or_else(|| decode::parse_mov(stream))
        .or_else(|| decode::parse_hlt(stream))
        .or_else(|| decode::parse_add(stream))
        .or_else(|| decode::parse_adc(stream))
        .or_else(|| decode::parse_sub(stream))
        .or_else(|| decode::parse_sbb(stream))
        .or_else(|| decode::parse_ana(stream))
        .or_else(|| decode::parse_xra(stream))
        .or_else(|| decode::parse_ora(stream))
        .or_else(|| decode::parse_cmp(stream))
        .or_else(|| decode::parse_rcc(stream))
        .or_else(|| decode::parse_pop(stream))
        .or_else(|| decode::parse_jcc(stream))
        .or_else(|| decode::parse_jmp(stream))
        .or_else(|| decode::parse_ccc(stream))
        .or_else(|| decode::parse_push(stream))
        .or_else(|| decode::parse_rst(stream))
        .or_else(|| decode::parse_ret(stream))
        .or_else(|| decode::parse_call(stream))
        .or_else(|| decode::parse_out(stream))
        .or_else(|| decode::parse_in(stream))
}
