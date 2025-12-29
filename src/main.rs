mod executable;
mod util;
mod vm;
mod devices{
    use crate::Disk;
    use crate::vm::{Machine};

    pub mod disk;
    pub mod audio;
    #[derive(Debug)]
    pub struct Device{
        pub driver: fn(machine: &mut Machine, command: i16),
        pub contents: RawDevice
    }
    #[derive(Debug)]
    pub enum RawDevice{
        Disk(Disk),
    }
}

use std::fmt::format;
use std::vec;
use prompted::input;
use vm::CommandType::{Exit, JumpNotZero, Load, Mov, Subf, NOP};
use crate::util::*;
use crate::executable::{Executable,Fn};
use crate::vm::{Bytecode,Machine,CommandType};
use crate::devices::disk::{Disk, DiskSection, DiskSectionType};
use tinyaudio::prelude::*;
fn main() {
    let mut main_fn =Fn::new("main".to_string());
    let mut exe=Executable::new();
    exe.add_constant(vec![4]);
    let nop_block = main_fn.add_block(vec![Bytecode::Command(NOP), Bytecode::Command(Exit)], false,false,false) as i16;
    main_fn.add_block(vec![
        Bytecode::Command(CommandType::Add), Bytecode::Int(1), Bytecode::Int(2),
        Bytecode::Command(Mov), Bytecode::Register(CommandType::R1), Bytecode::Register(CommandType::F1),
        Bytecode::Command(Subf), Bytecode::Register(CommandType::F1), Bytecode::Float(-5.6),
        Bytecode::Command(Load), Bytecode::Int(0), Bytecode::Register(CommandType::F2),
        Bytecode::Command(Subf), Bytecode::Register(CommandType::F1), Bytecode::Register(CommandType::F2),
        Bytecode::Command(JumpNotZero), Bytecode::Int(nop_block),
        Bytecode::Command(Exit)
    ], true,false,false);
    exe.add_fn(main_fn);
    let mut disk: Disk=vec![DiskSection{
        section_type: DiskSectionType::Entrypoint,
        id:0,
        data:vec![],
    }] as Disk;
    exe.build(0,&mut disk);
    let mut machine=Machine::new(true);
    machine.set_disk(disk);
    //machine.run();
    let params = OutputDeviceParameters {
        channels_count: 2,
        sample_rate: 44100,
        channel_sample_count: 4410,
    };

    let _device = run_output_device(params, {
        let mut clock = 0f32;
        move |data| {
            for samples in data.chunks_mut(params.channels_count) {
                clock = (clock + 1.0) % params.sample_rate as f32;
                let value =
                    (clock * 440.0*(clock%10.0)  / params.sample_rate as f32).sin();
                for sample in samples {
                    *sample = value;
                }
            }
        }
    })
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));
    //println!("{:?}");
}
