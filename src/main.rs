mod devices;
mod executable;
mod util;
mod vm;
use crate::devices::disk::{Disk, DiskSection, DiskSectionType};
use crate::executable::{Bytecode, Executable, Fn, Library};
use crate::vm::CommandType::*;
use crate::vm::{CommandType, Machine};
use devices::audio::load_wav;
use std::{fs, vec};
use util::{convert_float, unpack_float};
use vm::CommandType::{Exit, JumpNotZero, Load, Mov, NOP, Subf};
fn main() {
    let mut main_fn = Fn::new("main".to_string());
    let mut exe = Executable::new();
    let constant = exe.add_constant(vec![-5, 0]);
    let sound_file: Vec<i16> = load_wav(fs::read("sample.wav").unwrap().as_slice())
        .iter()
        .flat_map(|x| convert_float(*x))
        .collect();
    let file_size = sound_file.len() as i32;
    let mut another_fn = Fn::new("another_fn".to_string());
    let another_constant = exe.add_constant(vec![1, 2]);
    another_fn.add_block(
        vec![
            Bytecode::Command(Load),
            Bytecode::ConstantLoc(another_constant),
            Bytecode::Register(R2),
            Bytecode::Command(Add),
            Bytecode::ConstantLoc(another_constant),
            Bytecode::Int(1),
            Bytecode::Command(Load),
            Bytecode::Register(R1),
            Bytecode::Register(R1),
            Bytecode::Command(Add),
            Bytecode::Register(R1),
            Bytecode::Register(R2),
            Bytecode::Command(Push),
            Bytecode::Register(R1),
            Bytecode::Command(Return),
            Bytecode::Int(1),
        ],
        true,
    );
    exe.add_fn(another_fn);
    let do_nothing =
        main_fn.add_block(vec![Bytecode::Command(Jump), Bytecode::BlockLoc(-1)], false);
    main_fn.add_block(
        vec![
            Bytecode::Command(Call),
            Bytecode::FunctionRef("another_fn".to_string()),
            Bytecode::Int(0),
            Bytecode::Command(Pop),
            Bytecode::Register(R1),
            Bytecode::Command(Mov),
            Bytecode::Register(R1),
            Bytecode::Register(F1),
            Bytecode::Command(Addf),
            Bytecode::Float(0.5),
            Bytecode::Register(F1),
            Bytecode::Command(Store),
            Bytecode::ConstantLoc(constant),
            Bytecode::Register(F1),
            Bytecode::Command(Loadf),
            Bytecode::ConstantLoc(constant),
            Bytecode::Register(F1),
            Bytecode::Command(IO),
            Bytecode::Int(2),
            Bytecode::Int(0),
            Bytecode::Command(Call),
            Bytecode::FunctionRef("testLib::main".to_string()),
            Bytecode::Int(0),
            Bytecode::Command(Jump),
            Bytecode::BlockLoc(do_nothing),
        ],
        true,
    );
    exe.add_fn(main_fn);
    let mut test_lib = Library::new("testLib".to_string());
    let test_const = test_lib.add_constant(vec![6, 7]);
    let sound_sample = test_lib.add_constant(sound_file);
    test_lib.add_fn(Fn::new_with_blocks(
        "main".to_string(),
        vec![vec![
            Bytecode::Command(NOP),
            Bytecode::Command(Push),
            Bytecode::Int32(file_size),
            Bytecode::Command(Push),
            Bytecode::ConstantLoc(sound_sample),
            Bytecode::Command(Push),
            Bytecode::Int(9),
            Bytecode::Command(IO),
            Bytecode::Int(1),
            Bytecode::Int(6),
            Bytecode::Command(Return),
            Bytecode::Int(0),
        ]],
    ));
    test_lib.link(&mut exe);
    let mut disk: Disk = vec![DiskSection {
        section_type: DiskSectionType::Entrypoint,
        id: 0,
        data: vec![],
    }] as Disk;
    exe.build(0, &mut disk, true);
    let mut machine = Machine::new(true);
    machine.set_disk(disk);
    machine.run();
    //println!("{:?}");
}
