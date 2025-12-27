use crate::vm::{CommandType,Core};
use byteorder::{ByteOrder, LittleEndian};


pub fn resize_vec<T>(len:usize,vec:&mut Vec<T>,fill:T) where T: Clone{
    if vec.len()<=len{
        vec.resize(len,fill);
    }
}

pub fn flatten_vec(i: Vec<Vec<i16>>) -> Vec<i16> {
    i.into_iter().flat_map(|row| row).collect()
}
pub fn pack_float(f: f32) -> Vec<i16> {
    let mut rvec = vec![i16::MIN,0];
    rvec.extend(convert_float(f));
    rvec
}
pub fn unpack_float(bytes: &[i16]) -> Option<f32> {
    if bytes.len() != 2 {
        return None;
    }
    let i16_1 = bytes[0];
    let i16_2 = bytes[1];
    let mut rbytes = [0u8; 4];
    LittleEndian::write_i16(&mut rbytes[0..2], i16_1);
    LittleEndian::write_i16(&mut rbytes[2..4], i16_2);
    Some(f32::from_le_bytes(rbytes))
}
pub fn convert_float(f: f32) -> Vec<i16> {
    let native = f.to_ne_bytes();
    vec![
        LittleEndian::read_i16(&native[0..2]),
        LittleEndian::read_i16(&native[2..4]),
    ]
}
pub fn pop_stack(machine: &mut Core,bytes: i32)->Vec<f32> {
    let mut ret=Vec::new();
    for _i in 0..bytes {
        ret.push(machine.stack.pop(&mut machine.srp));
    }
    ret
}
pub fn convert_int_to_command(i: i16) -> CommandType {
    match i {
        32 => CommandType::Add,
        1 => CommandType::Sub,
        2 => CommandType::Mul,
        3 => CommandType::Div,
        4 => CommandType::Mod,
        5 => CommandType::Addf,
        6 => CommandType::Subf,
        7 => CommandType::Mulf,
        8 => CommandType::Divf,
        9 => CommandType::And,
        10 => CommandType::Not,
        11 => CommandType::Or,
        12 => CommandType::Xor,
        13 => CommandType::Push,
        14 => CommandType::Pop,
        15 => CommandType::Load,
        16 => CommandType::Store,
        17 => CommandType::Mov,
        19 => CommandType::Jump,
        20 => CommandType::JumpNotZero,
        21 => CommandType::Greater,
        22 => CommandType::LessThan,
        23 => CommandType::Exit,
        30 => CommandType::IP,
        31 => CommandType::SP,
        0 => CommandType::NOP,
        33 => CommandType::IO,
        34 => CommandType::Call,
        35=>CommandType::Return,
        36=>CommandType::JumpZero,
        _ => CommandType::NOP,
    }
}
pub fn pack_command(c: CommandType) -> i16 {
    match c {
        CommandType::Add => 32,
        CommandType::Sub => 1,
        CommandType::Mul => 2,
        CommandType::Div => 3,
        CommandType::Mod => 4,
        CommandType::Addf => 5,
        CommandType::Subf => 6,
        CommandType::Mulf => 7,
        CommandType::Divf => 8,
        CommandType::And => 9,
        CommandType::Not => 10,
        CommandType::Or => 11,
        CommandType::Xor => 12,
        CommandType::Push => 13,
        CommandType::Pop => 14,
        CommandType::Load => 15,
        CommandType::Store => 16,
        CommandType::Mov => 17,
        CommandType::Jump => 19,
        CommandType::JumpNotZero => 20,
        CommandType::Greater => 21,
        CommandType::LessThan => 22,
        CommandType::Exit => 23,
        CommandType::NOP => 0,
        CommandType::IO => 33,
        CommandType::Call=>34,
        CommandType::Return=>35,
        CommandType::JumpZero=>36,
        _ => 0,
    }
}
pub fn pack_register(r: CommandType) -> Vec<i16> {
    vec![
        i16::MIN,
        1,
        match r {
            CommandType::R1 => 1,
            CommandType::R2 => 2,
            CommandType::R3 => 3,
            CommandType::R4 => 4,
            CommandType::F1 => 5,
            CommandType::F2 => 6,
            CommandType::IP => 7,
            CommandType::SP => 8,
            CommandType::SRP => 9,
            _ => 0,
        },
    ]
}
pub fn convert_reg_byte_to_command(reg: i16, machine: &Core) -> f32 {
    match reg {
        1 => machine.r1 as f32,
        2 => machine.r2 as f32,
        3 => machine.r3 as f32,
        4 => machine.r4 as f32,
        5 => machine.f1,
        6 => machine.f2,
        7 => machine.ip as f32,
        8 => machine.stack.len() as f32,
        9=>machine.srp as f32,
        _ => -1.0,
    }
}
pub fn set_reg(reg: i16, machine: &mut Core, value: f32) {
    match reg {
        1 => machine.r1 = value as i16,
        2 => machine.r2 = value as i16,
        3 => machine.r3 = value as i16,
        4 => machine.r4 = value as i16,
        5 => machine.f1 = value,
        6 => machine.f2 = value,
        7 => machine.ip = value as usize,
        8 => machine.stack.resize(value as usize,&mut machine.srp),
        9=> machine.srp = value as usize,
        _ => (),
    }
}