use std::io::Write;

use crate::{coding::reader::Reader, instruction::Instruction};

mod parse;
pub mod reader;

pub fn encode(buffer: impl Write, _instructions: Vec<Instruction>) -> std::io::Result<()> {
    todo!()
}

pub fn decode<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    None.or_else(|| parse::parse_noop(stream))
        .or_else(|| parse::parse_lxi(stream))
        .or_else(|| parse::parse_stax(stream))
        .or_else(|| parse::parse_inx(stream))
        .or_else(|| parse::parse_inr(stream))
        .or_else(|| parse::parse_dcr(stream))
        .or_else(|| parse::parse_mvi(stream))
        .or_else(|| parse::parse_dad(stream))
        .or_else(|| parse::parse_ldax(stream))
        .or_else(|| parse::parse_dcx(stream))
        .or_else(|| parse::parse_rlc(stream))
        .or_else(|| parse::parse_rrc(stream))
        .or_else(|| parse::parse_ral(stream))
        .or_else(|| parse::parse_rar(stream))
        .or_else(|| parse::parse_shld(stream))
        .or_else(|| parse::parse_daa(stream))
        .or_else(|| parse::parse_lhld(stream))
        .or_else(|| parse::parse_cma(stream))
        .or_else(|| parse::parse_sta(stream))
        .or_else(|| parse::parse_stc(stream))
        .or_else(|| parse::parse_lda(stream))
        .or_else(|| parse::parse_cmc(stream))
        .or_else(|| parse::parse_mov(stream))
        .or_else(|| parse::parse_hlt(stream))
        .or_else(|| parse::parse_add(stream))
        .or_else(|| parse::parse_adc(stream))
        .or_else(|| parse::parse_sub(stream))
        .or_else(|| parse::parse_sbb(stream))
        .or_else(|| parse::parse_ana(stream))
        .or_else(|| parse::parse_xra(stream))
        .or_else(|| parse::parse_ora(stream))
        .or_else(|| parse::parse_cmp(stream))
        .or_else(|| parse::parse_rcc(stream))
        .or_else(|| parse::parse_pop(stream))
        .or_else(|| parse::parse_jcc(stream))
        .or_else(|| parse::parse_jmp(stream))
        .or_else(|| parse::parse_ccc(stream))
        .or_else(|| parse::parse_push(stream))
        .or_else(|| parse::parse_rst(stream))
        .or_else(|| parse::parse_ret(stream))
        .or_else(|| parse::parse_call(stream))
}
