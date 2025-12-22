mod executable;
mod util;
use std::vec;
use crate::CommandType::{Exit, JumpNotZero, Load, Mov, Subf, NOP};
use crate::DiskSectionType::Entrypoint;
use crate::util::*;
use crate::executable::{Executable,Fn};
fn main() {
    let mut main_fn =Fn::new("main".to_string());
    let mut exe=Executable::new();
    exe.add_constant(vec![4]);
    let nop_block = main_fn.add_block(vec![Bytecode::Command(NOP), Bytecode::Command(Exit)], false) as i16;
    main_fn.add_block(vec![
        Bytecode::Command(CommandType::Add), Bytecode::Int(1), Bytecode::Int(2),
        Bytecode::Command(Mov), Bytecode::Register(CommandType::R1), Bytecode::Register(CommandType::F1),
        Bytecode::Command(Subf), Bytecode::Register(CommandType::F1), Bytecode::Float(-5.6),
        Bytecode::Command(Load), Bytecode::Int(0), Bytecode::Register(CommandType::F2),
        Bytecode::Command(Subf), Bytecode::Register(CommandType::F1), Bytecode::Register(CommandType::F2),
        Bytecode::Command(JumpNotZero), Bytecode::Int(nop_block),
        Bytecode::Command(Exit)
    ], true);
    exe.add_fn(main_fn);
    let mut disk: Disk=vec![DiskSection{
        section_type: Entrypoint,
        id:0,
        data:vec![],
    }] as Disk;
    exe.build(0,&mut disk);
    dbg!(&disk);
    println!("{:?}", run_bytecode(&mut disk,true));
}

enum Bytecode{
    Command(CommandType),
    Register(CommandType),
    Float(f32),
    Int(i16),
}
fn run_bytecode(disk: &mut Disk,debug:bool) -> Core {
    let mut machine: Core = Core::new();
    machine.debug=debug;
    if disk[0].data.len() >= 256 {
        machine.memory = disk[0].data[0..256].to_vec();
    } else {
        machine.memory = disk[0].data.clone();
    }
    while machine.on {
        let byte = convert_int_to_command(take_bytes(&mut machine, 1)[0] as i16);
        match byte {
            CommandType::Add => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] + args[1]) as i16;
                if machine.debug{
                    println!("Add {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Sub => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] - args[1]) as i16;
                if machine.debug{
                    println!("Sub {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Mul => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] * args[1]) as i16;
                if machine.debug{
                    println!("Mul {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Div => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] / args[1]) as i16;
                if machine.debug{
                    println!("Div {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Greater => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] > args[1]) as i16;
                if machine.debug{
                    println!("GreaterThan {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Addf => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] + args[1];
                if machine.debug{
                    println!("Addf {} {} -> {}",args[0],args[1],machine.f1);
                }
            }
            CommandType::Subf => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] - args[1];
                if machine.debug{
                    println!("Subf {} {} -> {}",args[0],args[1],machine.f1);
                }
            }
            CommandType::Mulf => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] * args[1];
                if machine.debug{
                    println!("Mulf {} {} -> {}",args[0],args[1],machine.f1);
                }
            }
            CommandType::Divf => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] / args[1];
                if machine.debug{
                    println!("Divf {} {} -> {}",args[0],args[1],machine.f1);
                }
            }
            CommandType::Mod => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] % args[1]) as i16;
                if machine.debug{
                    println!("Modulo {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Pop => {
                let val = machine.stack.pop(&mut machine.srp);
                let reg=take_registers(&mut machine, 1)[0];
                set_reg(reg, &mut machine, val);
                if machine.debug{
                    println!("Pop {} -> R{}",val,reg);
                }
            }
            CommandType::LessThan => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] < args[1]) as i16;
                if machine.debug{
                    println!("LessThan {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Jump => {
                let addr = take_bytes(&mut machine, 1)[0];
                machine.ip = addr as usize;
                if machine.debug{
                    println!("Jump {}",addr);
                }
            }
            CommandType::And => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = args[0] as i16 & args[1] as i16;
                if machine.debug{
                    println!("And {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Or => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = args[0] as i16 | args[1] as i16;
                if machine.debug{
                    println!("Or {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Not => {
                let args = take_bytes(&mut machine, 1);
                machine.r1 = !(args[0] as i16);
                if machine.debug{
                    println!("Not {} -> {}",args[0],machine.r1);
                }
            }
            CommandType::Xor => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = args[0] as i16 ^ (args[1] as i16);
                if machine.debug{
                    println!("Xor {} {} -> {}",args[0],args[1],machine.r1);
                }
            }
            CommandType::Push => {
                let args = take_bytes(&mut machine, 1);
                machine.stack.push(args[0],&mut machine.srp);
                if machine.debug{
                    println!("Push {}",args[0]);
                }
            }
            CommandType::Mov => {
                let args = take_bytes(&mut machine, 1);
                let reg=take_registers(&mut machine, 1)[0];
                set_reg(reg, &mut machine, args[0]);
                if machine.debug{
                    println!("Mov {} -> R{}",args[0],reg);
                }
            }
            CommandType::JumpNotZero => {
                let args = take_bytes(&mut machine, 2);
                if args[1] != 0.0 {
                    machine.ip = args[0] as usize;
                }
                if machine.debug{
                    println!("JumpNotZero {} {}",args[0],args[1]);
                }

            }
            CommandType::Load => {
                let args = take_bytes(&mut machine, 1);
                let val = machine.memory[args[0] as usize] as f32;
                let reg=take_registers(&mut machine, 1)[0];
                set_reg(reg, &mut machine, val);
                if machine.debug{
                    println!("Load %{} -> R{}",args[0],reg);
                }
            }
            CommandType::Store => {
                let args = take_bytes(&mut machine, 2);
                if args[1].fract() == 0.0 {
                machine.memory[args[0] as usize] = args[1] as i16;}else{
                    let f = convert_float(args[1]);
                    machine
                        .memory
                        .splice(args[0] as usize..args[0] as usize + f.len(), f);
                }
                if machine.debug{
                    println!("Store {} -> %{}",args[1],args[0]);
                }
            }
            CommandType::Exit => {
                machine.on = false;
                if machine.debug{
                    println!("Exit");
                }
            }
            CommandType::Loadf => {
                let args = take_bytes(&mut machine, 1);
                let val = unpack_float(&machine.memory[args[0] as usize..args[0] as usize + 1usize])
                    .expect(&format!("Couldn't get float at memory address {}", args[0]));
                let reg=take_registers(&mut machine, 1)[0];
                set_reg(reg, &mut machine, val);
                if machine.debug{
                    println!("Loadf %{} -> R{}",args[0],reg);
                }
            }
            CommandType::IO => {
                //io(device,command), driverags are on stack
                let args = take_bytes(&mut machine, 2);
                if machine.debug{
                    println!("IO {} {}",args[0],args[1]);
                }
                match args[0] as i16 {
                    0 => {
                        //disk
                        match args[1] as i16 {
                            0 => {
                                //read(section,addr,len)
                                let cargs = pop_stack(&mut machine, 3);
                                let range = (cargs[1] as usize)..(cargs[1] + cargs[2]) as usize;
                                machine.stack.extend(
                                    disk[cargs[0] as usize].data[range]
                                        .to_vec()
                                        .iter()
                                        .map(|x| *x as f32).collect(),&mut machine.srp
                                );
                                if machine.debug{
                                    println!("IO.disk.read {} {} {}",cargs[0],cargs[1],cargs[2]);
                                }
                            }
                            1 => {
                                //write(section,addr,byte)
                                let cargs = pop_stack(&mut machine, 3);
                                disk[cargs[0] as usize].data[cargs[1] as usize] = cargs[2] as i16;
                                if machine.debug{
                                    println!("IO.disk.write {} -> disk.%[{} {}]",cargs[2],cargs[0],cargs[1]);
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            CommandType::Call=>{
                //call(fnptr,argcount,...args)
                let cargs = take_bytes(&mut machine, 2);
                let fnargs= take_bytes(&mut machine, cargs[1] as i16);
                machine.stack.push(machine.ip as f32,&mut machine.srp);
                for i in &fnargs{
                    machine.stack.push(*i,&mut machine.srp);
                }
                machine.ip=cargs[0] as usize;
                if machine.debug{
                    println!("Call {} Array[{},{:?}]",cargs[0],cargs[1],fnargs);
                }
            }
            CommandType::Return=>{
                //return(returned_byte_count)
                let args= take_bytes(&mut machine, 1);
                machine.ip=machine.stack.remove(machine.srp-(args[0] as usize),&mut machine.srp) as usize;
                if machine.debug{
                    println!("Return {}",args[0]);
                }
            }
            CommandType::NOP => {
                if machine.debug{
                    println!("NOP");
                }
            }
            _ => {}
        }
    }
    machine
}
fn take_bytes(core: &mut Core, bytecount: i16) -> Vec<f32> {
    let mut offset = core.ip;
    let mut real_byte_count = 0;
    let mut bytes: Vec<f32> = Vec::new();
    for i in 0..bytecount {
        let byte = core.memory[offset + i as usize];
        if byte == i16::MIN {
            match core.memory[offset+i as usize+1]{
                0=>{
            bytes.push(
                unpack_float(&[
                    core.memory[offset + 2 + i as usize],
                    core.memory[offset + 3 + i as usize],
                ])
                .expect("Couldn't convert bytes from i16 to float"),
            );
            offset += 3;
            real_byte_count += 4;
            }
                1=>{
                    bytes.push(convert_reg_byte_to_command(
                        core.memory[offset + 2 + i as usize],
                        core,
                    ));
                    offset += 2;
                    real_byte_count += 3;
                }
                _=>{}
            }
        } else {
            bytes.push(byte as f32);
            real_byte_count += 1;
        }
    }
    core.ip += real_byte_count;
    bytes
}
fn take_registers(core: &mut Core, count: i16) -> Vec<i16> {
    let mut bytes: Vec<i16> = Vec::new();
    for i in 0..count {
        let byte = core.memory[core.ip + 2 + (i * 3) as usize];
        bytes.push(byte);
    }
    core.ip += (count * 3) as usize;
    bytes
}
#[derive(Debug)]
struct Core {
    ip: usize,
    stack: Stack,
    r1: i16,
    r2: i16,
    r3: i16,
    r4: i16,
    f1: f32,
    f2: f32,
    srp: usize,
    on: bool,
    debug:bool,
    memory: Vec<i16>,
}
impl Core {
    fn new() -> Core {
        Core {
            ip: 0,
            stack: Stack::new(),
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            f1: 0.0,
            f2: 0.0,
            srp:2,
            on: true,
            debug:false,
            memory: Vec::new(),
        }
    }
}
#[derive(Debug)]
struct Stack{
    data: Vec<f32>,
}
impl Stack {
    fn new() -> Stack {
        Stack{
            data: Vec::new(),
        }
    }
    fn len(&self) -> usize{
        self.data.len()
    }
    fn push(&mut self, x: f32,srp:&mut usize) {
        if *srp>=self.data.len(){
            self.data.resize(*srp,0.0);
        }
        self.data.insert(*srp,x);
        *srp+=1;
    }
    fn pop(&mut self, srp:&mut usize) -> f32 {
        self.remove(*srp -1,srp)
    }
    fn extend(&mut self, data: Vec<f32>,srp:&mut usize) {
        for (i,item) in data.iter().enumerate(){
            self.data.insert(*srp+i,*item);
        }
        *srp+=data.len();
    }
    fn remove(&mut self, index: usize,srp:&mut usize)-> f32 {
        *srp-=1;
        self.data.remove(index)
    }
    fn resize(&mut self, size: usize,srp:&mut usize){
        if size<=self.data.len() {
            *srp=size;
        }
        self.data.resize(size,0.0);
    }
}
type Disk = Vec<DiskSection>;
#[derive(Debug,Clone)]
struct DiskSection {
    section_type: DiskSectionType,
    data: Vec<i16>,
    id: i16,
}
#[derive(Debug, PartialEq,Clone)]
enum DiskSectionType {
    Entrypoint,
    Libary,
    Code,
    Loader,
    Data,
}
#[repr(u8)]
#[derive(Debug,Copy,Clone)]
enum CommandType {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Addf,
    Subf,
    Mulf,
    Divf,
    And,
    Not,
    Or,
    Xor,
    Push,
    Pop,
    Load,
    Store,
    Mov,
    Jump,
    JumpNotZero,
    Greater,
    LessThan,
    Exit,
    R1,
    R2,
    R3,
    R4,
    F1,
    F2,
    IP,
    SP,
    SRP,
    NOP,
    IO,
    Loadf,
    Call,
    Return
}

