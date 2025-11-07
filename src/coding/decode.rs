use std::ops::Range;

use crate::{
    coding::reader::Reader,
    instruction::{
        Condition, Data16, Instruction, Register, RegisterPair, RegisterPairIndirect,
        RegisterPairOrStatus, RestartNumber,
    },
};

fn is_eq_masked(byte: u8, expected: u8, mask: u8) -> bool {
    byte & mask == expected & mask
}

fn extract_bits(byte: u8, range: Range<u8>) -> u8 {
    let shifted = byte >> range.start;
    // Magic formula from here:
    // https://users.rust-lang.org/t/calculating-dynamic-bitmasks-for-function-generation/67148/3
    let mask = (1 << range.len()) - 1;

    // u8::from_str_radix(src, radix)

    shifted & mask
}

pub fn parse_noop<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_0000, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    Some(Instruction::Nop)
}

pub fn parse_lxi<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    let bytes = stream.peek_n(3)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_0001, 0b1100_1111) {
        return None;
    };

    let rp: RegisterPair = extract_bits(opcode, 4..6)
        .try_into()
        .expect("Register pairs cover all 2 bit numbers");

    let data = Data16 {
        low: bytes[1],
        high: bytes[2],
    };

    stream.skip_n(3);

    return Some(Instruction::Lxi(rp, data));
}

pub fn parse_stax<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    let bytes = stream.peek_n(1)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_0010, 0b1100_1111) {
        return None;
    };

    let rp: RegisterPairIndirect = extract_bits(opcode, 4..6).try_into().ok()?;

    stream.skip_n(1);

    return Some(Instruction::Stax(rp));
}

pub fn parse_inx<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    let bytes = stream.peek_n(1)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_0011, 0b1100_1111) {
        return None;
    };

    let rp: RegisterPair = extract_bits(opcode, 4..6)
        .try_into()
        .expect("Register pairs cover all 2 bit numbers");

    stream.skip_n(1);

    return Some(Instruction::Inx(rp));
}

pub fn parse_inr<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    let bytes = stream.peek_n(1)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_0100, 0b1100_0111) {
        return None;
    };

    let rp: Register = extract_bits(opcode, 3..6)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(1);

    return Some(Instruction::Inr(rp));
}

pub fn parse_dcr<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    let bytes = stream.peek_n(1)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_0101, 0b1100_0111) {
        return None;
    };

    let rp: Register = extract_bits(opcode, 3..6)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(1);

    return Some(Instruction::Dcr(rp));
}

pub fn parse_mvi<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    let bytes = stream.peek_n(2)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_0110, 0b1100_0111) {
        return None;
    };

    let rp: Register = extract_bits(opcode, 3..6)
        .try_into()
        .expect("Registers cover all 3 bit numbers");
    let data = bytes[1];

    stream.skip_n(2);

    return Some(Instruction::Mvi(rp, data));
}

pub fn parse_dad<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_1001, 0b1100_1111) {
        return None;
    };

    let rp: RegisterPair = extract_bits(opcode, 4..6)
        .try_into()
        .expect("Register pairs cover all 2 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Dad(rp));
}

pub fn parse_ldax<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_1010, 0b1100_1111) {
        return None;
    };

    let rp: RegisterPairIndirect = extract_bits(opcode, 4..6).try_into().ok()?;

    stream.skip_n(LEN);

    return Some(Instruction::Ldax(rp));
}

pub fn parse_dcx<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_1011, 0b1100_1111) {
        return None;
    };

    let rp: RegisterPair = extract_bits(opcode, 4..6)
        .try_into()
        .expect("Register pairs cover all 2 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Dcx(rp));
}

pub fn parse_rlc<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_0111, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Rlc);
}

pub fn parse_rrc<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0000_1111, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Rrc);
}

pub fn parse_ral<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0001_0111, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Ral);
}

pub fn parse_rar<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0001_1111, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Rar);
}

pub fn parse_shld<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 3;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0010_0010, 0b1111_1111) {
        return None;
    };

    let data = Data16::new(bytes[1], bytes[2]);

    stream.skip_n(LEN);

    return Some(Instruction::Shld(data.into()));
}

pub fn parse_daa<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0010_0111, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Daa);
}

pub fn parse_lhld<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 3;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0010_1010, 0b1111_1111) {
        return None;
    };

    let data = Data16::new(bytes[1], bytes[2]);

    stream.skip_n(LEN);

    return Some(Instruction::Lhld(data.into()));
}

pub fn parse_cma<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0010_1010, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Cma);
}

pub fn parse_sta<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 3;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0011_0010, 0b1111_1111) {
        return None;
    };

    let data = Data16::new(bytes[1], bytes[2]);

    stream.skip_n(LEN);

    return Some(Instruction::Sta(data.into()));
}

pub fn parse_stc<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0011_0111, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Stc);
}

pub fn parse_lda<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 3;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0011_1010, 0b1111_1111) {
        return None;
    };

    let data = Data16::new(bytes[1], bytes[2]);

    stream.skip_n(LEN);

    return Some(Instruction::Lda(data.into()));
}

pub fn parse_cmc<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0011_1111, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Cmc);
}

pub fn parse_mov<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0100_0000, 0b1100_0000) {
        return None;
    };

    let ddd: Register = extract_bits(opcode, 3..6)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Mov(ddd, sss));
}

pub fn parse_hlt<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b0111_0110, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Hlt);
}

pub fn parse_add<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1000_0000, 0b1111_1000) {
        return None;
    };

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Add(sss));
}

pub fn parse_adc<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1001_0000, 0b1111_1000) {
        return None;
    };

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Adc(sss));
}

pub fn parse_sub<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1001_0000, 0b1111_1000) {
        return None;
    };

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Sub(sss));
}

pub fn parse_sbb<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1001_1000, 0b1111_1000) {
        return None;
    };

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Sbb(sss));
}

pub fn parse_ana<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1010_0000, 0b1111_1000) {
        return None;
    };

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Ana(sss));
}

pub fn parse_xra<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1010_1000, 0b1111_1000) {
        return None;
    };

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Xra(sss));
}

pub fn parse_ora<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1011_0000, 0b1111_1000) {
        return None;
    };

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Ora(sss));
}

pub fn parse_cmp<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1011_1000, 0b1111_1000) {
        return None;
    };

    let sss: Register = extract_bits(opcode, 0..3)
        .try_into()
        .expect("Registers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Cmp(sss));
}

pub fn parse_rcc<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_0000, 0b1100_0111) {
        return None;
    };

    let cc: Condition = extract_bits(opcode, 3..6).try_into().ok()?;

    stream.skip_n(LEN);

    return Some(Instruction::Rcc(cc));
}

pub fn parse_pop<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_0001, 0b1100_1111) {
        return None;
    };

    let rp: RegisterPairOrStatus = extract_bits(opcode, 4..6)
        .try_into()
        .expect("Register pair or statuses cover all 2 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Pop(rp));
}

pub fn parse_jcc<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 3;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_0010, 0b1100_0111) {
        return None;
    };

    let cc: Condition = extract_bits(opcode, 3..6).try_into().ok()?;

    let addr = Data16 {
        low: bytes[1],
        high: bytes[2],
    };

    stream.skip_n(LEN);

    return Some(Instruction::Jcc(cc, addr.into()));
}

pub fn parse_jmp<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 3;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_0011, 0b1111_1111) {
        return None;
    };

    let addr = Data16 {
        low: bytes[1],
        high: bytes[2],
    };

    stream.skip_n(LEN);

    return Some(Instruction::Jmp(addr.into()));
}

pub fn parse_ccc<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 3;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_0100, 0b1100_0111) {
        return None;
    };

    let cc: Condition = extract_bits(opcode, 3..6).try_into().ok()?;

    let addr = Data16 {
        low: bytes[1],
        high: bytes[2],
    };

    stream.skip_n(LEN);

    return Some(Instruction::Ccc(cc, addr.into()));
}

pub fn parse_push<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_0101, 0b1100_1111) {
        return None;
    };

    let rp: RegisterPairOrStatus = extract_bits(opcode, 4..6).try_into().ok()?;

    stream.skip_n(LEN);

    return Some(Instruction::Push(rp));
}

// ADI ACI SUI SBI ANI XRI ORI CPI... (I skipped these)

pub fn parse_rst<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_0111, 0b1100_0111) {
        return None;
    };

    let n: RestartNumber = extract_bits(opcode, 3..6)
        .try_into()
        .expect("Restart numbers cover all 3 bit numbers");

    stream.skip_n(LEN);

    return Some(Instruction::Rst(n));
}

pub fn parse_ret<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 1;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_1001, 0b1111_1111) {
        return None;
    };

    stream.skip_n(LEN);

    return Some(Instruction::Ret);
}

pub fn parse_call<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 3;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1100_1101, 0b1111_1111) {
        return None;
    };

    let addr = Data16 {
        low: bytes[1],
        high: bytes[2],
    };

    stream.skip_n(LEN);

    return Some(Instruction::Call(addr.into()));
}

pub fn parse_out<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 2;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1101_0011, 0b1111_1111) {
        return None;
    };

    let port = bytes[1];

    stream.skip_n(LEN);

    return Some(Instruction::Out(port));
}

pub fn parse_in<'a>(stream: &mut Reader<'a>) -> Option<Instruction> {
    static LEN: usize = 2;
    let bytes = stream.peek_n(LEN)?;
    let opcode = bytes[0];
    if !is_eq_masked(opcode, 0b1101_1011, 0b1111_1111) {
        return None;
    };

    let port = bytes[1];

    stream.skip_n(LEN);

    return Some(Instruction::In(port));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bits() {
        assert_eq!(extract_bits(0b1101_0011, 2..6), 0b0100)
    }
}
