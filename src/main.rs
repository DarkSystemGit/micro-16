use std::collections::HashMap;
use byteorder::{ByteOrder, LittleEndian};
use std::vec;
use crate::CommandType::{Exit, Jump, JumpNotZero, Load, Mov, Push, Subf, IO, NOP};
use crate::DiskSectionType::{Code, Entrypoint, Libary};

fn main() {
    let mut mainfn=Fn::new("main".to_string());
    let mut exe=Exectutable::new();
    exe.add_constant(vec![4]);
    let nopblock=mainfn.add_block(vec![Bytecode::Command(NOP),Bytecode::Command(Exit)],false) as i16;
    mainfn.add_block(vec![
                          Bytecode::Command(CommandType::Add),Bytecode::Int(1),Bytecode::Int(2),
                          Bytecode::Command(Mov),Bytecode::Register(CommandType::R1),Bytecode::Register(CommandType::F1),
                          Bytecode::Command(Subf),Bytecode::Register(CommandType::F1),Bytecode::Float(-5.6),
                          Bytecode::Command(Load),Bytecode::Int(0),Bytecode::Register(CommandType::F2),
                          Bytecode::Command(Subf),Bytecode::Register(CommandType::F1),Bytecode::Register(CommandType::F2),
                          Bytecode::Command(JumpNotZero),Bytecode::Int(nopblock),
                          Bytecode::Command(Exit)
    ],true);
    exe.add_fn(mainfn);
    let mut disk: Disk=vec![DiskSection{
        section_type: Entrypoint,
        id:0,
        data:vec![],
    }] as Disk;
    exe.build(0,&mut disk);
    //dbg!(disk);
    //println!("{:?}", run_bytecode(&mut disk));
}
#[derive(Debug,Clone)]
struct Exectutable{
    data: Vec<i16>,
    data_offsets: Vec<i16>,
    fns: Vec<Fn>,
    loader: Vec<i16>
}
impl Exectutable{
    fn new() -> Exectutable {
        Exectutable {
            data: Vec::new(),
            data_offsets: Vec::new(),
            fns: Vec::new(),
            loader: flatten_vec(vec![
                gen_io_read(256,0,3),
                vec![pack_command(CommandType::Pop)],pack_register(CommandType::R2),
                vec![pack_command(CommandType::Pop)],pack_register(CommandType::R3),
                vec![pack_command(CommandType::Pop)],pack_register(CommandType::R4),
                //R2=data sector,R3=data len, R4=Bytecode len

            ])
        }
    }
    fn set_loader(&mut self,loader: Vec<i16>) {
        if loader.len()>256{
            println!("Oversized executable loader");
        }
        self.loader=loader;
    }
    fn add_constant(&mut self,constant: Vec<i16>)->usize{
        let offset=self.data.len();
        self.data.extend(constant);
        self.data_offsets.push(offset as i16);
        offset
    }
    fn add_fn(&mut self, mut data: Fn) ->usize{
        let id=self.fns.len();
        data.id=id;
        self.fns.push(data);
        0
    }
    fn build(mut self,mut offset: usize, disk: &mut Disk) {
        let mut bytecode: Vec<i16>=vec![];
        let mut fnmap: HashMap<usize,usize>=HashMap::new();
        //loader
        offset+=255;
        //headers
        offset+=5;
        let mut mainloc=0;
        let data_sec=offset as i16+self.fns.iter_mut().enumerate().fold(
            offset,
            |acc,(i,func)|{
                if func.name=="main"{
                    mainloc=acc;
                }
                fnmap.insert(i,acc);
                acc+func.blocks.iter().map(|b|b.len()-1).sum::<usize>() as usize
            }
        ) as i16;
        //bytecode.push(as i16);
        for (i,func) in self.fns.iter_mut().enumerate(){
            let mut blockmap: HashMap<usize,usize>=HashMap::new();
            func.blocks.iter().enumerate().fold(
                fnmap[&i],
                |acc,(i,b)| {
                    blockmap.insert(i,acc);
                    acc+b.len()
                }
            );

            for (i,block) in func.blocks.iter_mut().enumerate(){
                let jumps=&func.jumps[i];
                let constantusage=&func.constantaccess[i];
                jumps.iter().for_each(|j|{
                    let jumploc=block[*j + 1] as usize;
                    block[j+1]=match convert_int_to_command(block[*j]){
                        CommandType::Jump=>{
                             blockmap[&(jumploc)]
                        },
                        CommandType::JumpNotZero=>{
                            blockmap[&(jumploc)]
                        },
                        CommandType::Call=>{
                            fnmap[&jumploc]
                        },
                        _=>0
                    } as i16;
                });
                constantusage.iter().for_each(|constantloc|{
                    let constant=block[*constantloc];
                    block[*constantloc]= data_sec + self.data_offsets[constant as usize];
                });
                bytecode.extend(block.iter().map(|x|*x));
            }
        }
        Self::insert_bytecode_into_disk(&self,disk,bytecode,offset,mainloc);
    }
    fn insert_bytecode_into_disk(&self,disk: &mut Disk,bytecode: Vec<i16>,mut offset:usize,entrypoint:usize) {
        //Executable Structure
        //-data sector
        //-data len
        //-bytecode len
        //bytecode
        //data

        //(total exe code len/max sector data).ceil()
        let sectors=((offset+bytecode.len()) as f32/i16::MAX as f32).ceil() as usize;
        let headers=vec![sectors,self.data.len(),bytecode.len()+2];
        let insertion_jump=vec![pack_command(Jump),entrypoint as i16];
        let executable= flatten_vec(vec![headers.iter().map(|x|*x as i16).collect(), insertion_jump, bytecode]);
        //remove headers for these calcs
        offset-=5;
        let base_sector=(offset as f32/i16::MAX as f32).floor() as usize;
        let bsector_offset=(offset as f32%i16::MAX as f32) as usize;
        let data_sector_count=(self.data.len() as f32/i16::MAX as f32).ceil() as usize;
        for i in base_sector..sectors{
            if i==base_sector{
                let insert_len=match executable.len()<i16::MAX as usize{
                    true=>executable.len(),
                    false=>(i16::MAX as usize)
                };
                resize_vec(bsector_offset+insert_len,&mut disk[i].data,0);
                disk[i].data.splice(bsector_offset..,executable[0..insert_len].to_vec());
            }else{
                disk[i].section_type =match disk[base_sector].section_type {
                    Entrypoint=>Code,
                    DiskSectionType::Libary=>Libary,
                    _=>Code,
                };
                let sector_start=(i16::MAX as usize)*(i-base_sector);
                let sector_end=(i16::MAX as usize)*(i-base_sector+1);
                disk[i].data=executable[sector_start..sector_end].to_vec();
            }
        }

        for i in sectors..sectors+data_sector_count{
            resize_vec(i+1,disk,DiskSection{
                section_type: DiskSectionType::Data,
                id:-1,
                data:vec![],
            });
            let iteration=i-sectors;
            let data_start=iteration*i16::MAX as usize;
            let data_end=match self.data.len()<(iteration+1)*i16::MAX as usize{
                false=>(iteration+1)*i16::MAX as usize,
                true=>self.data.len()
            };
            disk[i]=DiskSection{
                section_type: DiskSectionType::Data,
                id:i as i16,
                data:self.data[data_start..data_end].to_vec(),
            };
        }
        let mut loader=self.loader.clone();
        resize_vec(256,&mut loader,0);
        disk[0].data.splice(0..255,loader);
        dbg!(disk);
    }

}
fn gen_io_read(addr:i16,sector:i16,len:i16)->Vec<i16>{
    vec![pack_command(Push),len,pack_command(Push),sector,pack_command(Push),addr,pack_command(IO),0,0]
}
fn resize_vec<T>(len:usize,vec:&mut Vec<T>,fill:T) where T: Clone{
    if vec.len()<=len{
        vec.resize(len,fill);
    }
}
#[derive(Debug,Clone)]
struct Fn{
    name: String,
    blocks: Vec<Vec<i16>>,
    jumps: Vec<Vec<usize>>,
    constantaccess: Vec<Vec<usize>>,
    id: usize,
    loc: usize,
}
impl Fn{
    fn new(name: String)->Fn{
        Fn{
            name:name,
            blocks: vec![vec![19,0]],
            jumps: vec![vec![0]],
            constantaccess: vec![vec![]],
            id:0,
            loc: 0
        }
    }
    fn add_block(&mut self,block: Vec<Bytecode>,entrypoint:bool)->usize{
        let mut jumps: Vec<usize>=vec![];
        let mut constant_usages: Vec<usize>=vec![];
        let mut loc=0;
        self.blocks.push(block.iter().flat_map(|bytecode|{
            let r=match bytecode{
                Bytecode::Command(c)=>{
                    match c{
                        CommandType::Loadf=>{constant_usages.push(loc+1)}
                        CommandType::Load=>{constant_usages.push(loc+1)}
                        CommandType::Store=>{constant_usages.push(loc+1)}
                        CommandType::Jump=>{jumps.push(loc)}
                        CommandType::JumpNotZero=>{jumps.push(loc)}
                        CommandType::Call=>{jumps.push(loc)}
                        _=>{}
                    };
                    vec![pack_command(*c)]
                },
                Bytecode::Float(f)=>pack_float(*f),
                Bytecode::Int(i)=>vec![*i],
                Bytecode::Register(r)=>pack_register(*r),
            };
            loc+=r.len();
            r
        }).collect());
        self.jumps.push(jumps);
        self.constantaccess.push(constant_usages);
        self.blocks[0][1]=(self.blocks.len()-1) as i16;
        self.blocks.len()-1
    }
}
enum Bytecode{
    Command(CommandType),
    Register(CommandType),
    Float(f32),
    Int(i16),
}
fn flatten_vec(i: Vec<Vec<i16>>) -> Vec<i16> {
    i.into_iter().flat_map(|row| row).collect()
}
fn pack_float(f: f32) -> Vec<i16> {
    let mut rvec = vec![i16::MIN];
    rvec.extend(convert_float(f));
    rvec
}
fn unpack_float(bytes: &[i16]) -> Option<f32> {
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
fn convert_float(f: f32) -> Vec<i16> {
    let native = f.to_ne_bytes();
    vec![
        LittleEndian::read_i16(&native[0..2]),
        LittleEndian::read_i16(&native[2..4]),
    ]
}
fn pop_stack(machine: &mut Core,bytes: i32)->Vec<f32> {
    let mut ret=Vec::new();
    for _i in 0..bytes {
        ret.push(machine.stack.pop().unwrap_or(0.0));
    }
    return ret;
}
fn run_bytecode(disk: &mut Disk) -> Core {
    let mut machine: Core = Core::new();
    if disk[0].data.len() >= 256 {
        machine.memory = disk[0].data[0..256].to_vec();
    } else {
        machine.memory = disk[0].data.clone();
    }
    while machine.on {
        let byte = convert_int_to_command(take_bytes(&mut machine, 1)[0] as i16);
        dbg!(&byte, machine.ip);
        match byte {
            CommandType::Add => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] + args[1]) as i16;
            }
            CommandType::Sub => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] - args[1]) as i16;
            }
            CommandType::Mul => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] * args[1]) as i16;
            }
            CommandType::Div => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] / args[1]) as i16;
            }
            CommandType::Greater => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] > args[1]) as i16;
            }
            CommandType::Addf => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] + args[1];
            }
            CommandType::Subf => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] - args[1];
            }
            CommandType::Mulf => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] - args[1];
            }
            CommandType::Divf => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] / args[1];
            }
            CommandType::Mod => {
                let args = take_bytes(&mut machine, 2);
                machine.f1 = args[0] % args[1];
            }
            CommandType::Pop => {
                let val = machine.stack.pop().expect("stack underflow");
                set_reg(take_registers(&mut machine, 1)[0], &mut machine, val);
            }
            CommandType::LessThan => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] < args[1]) as i16;
            }
            CommandType::Jump => {
                let addr = take_bytes(&mut machine, 1)[0];
                machine.ip = addr as usize;
            }
            CommandType::And => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] as i16 & args[1] as i16);
            }
            CommandType::Or => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = (args[0] as i16 | args[1] as i16);
            }
            CommandType::Not => {
                let args = take_bytes(&mut machine, 1);
                machine.r1 = !(args[0] as i16);
            }
            CommandType::Xor => {
                let args = take_bytes(&mut machine, 2);
                machine.r1 = args[0] as i16 ^ (args[1] as i16);
            }
            CommandType::Push => {
                let args = take_bytes(&mut machine, 1);
                machine.stack.push(args[0]);
            }
            CommandType::Mov => {
                let args = take_bytes(&mut machine, 1);
                set_reg(take_registers(&mut machine, 1)[0], &mut machine, args[0]);
            }
            CommandType::JumpNotZero => {
                let args = take_bytes(&mut machine, 2);
                if args[1] != 0.0 {
                    machine.ip = args[0] as usize;
                }
            }
            CommandType::Load => {
                let args = take_bytes(&mut machine, 1);
                let val = machine.memory[args[0] as usize] as f32;
                set_reg(take_registers(&mut machine, 1)[0], &mut machine, val);
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
            }
            CommandType::Exit => {
                machine.on = false;
            }
            CommandType::Loadf => {
                let args = take_bytes(&mut machine, 1);
                let val = unpack_float(&machine.memory[args[0] as usize..args[0] as usize + 1 as usize])
                    .expect(&format!("Couldn't get float at memory address {}", args[0]));
                set_reg(take_registers(&mut machine, 1)[0], &mut machine, val);
            }
            CommandType::IO => {
                //io(device,command), driverags are on stack
                let args = take_bytes(&mut machine, 2);
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
                                        .map(|x| *x as f32),
                                );
                            }
                            1 => {
                                //write(section,addr,byte)
                                let cargs = pop_stack(&mut machine, 3);
                                disk[cargs[0] as usize].data[cargs[1] as usize] = cargs[2] as i16;
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
                machine.stack.push(machine.ip as f32);
                for i in fnargs{
                    machine.stack.push(i);
                }
                machine.ip=cargs[0] as usize;
            }
            CommandType::Return=>{
                //return(returned_byte_count)
                let args= take_bytes(&mut machine, 1);
                machine.ip=machine.stack.remove(machine.stack.len()-(args[0] as usize)) as usize;
            }
            CommandType::NOP => {}
            _ => {}
        }
    }
    return machine;
}
fn take_bytes(core: &mut Core, bytecount: i16) -> Vec<f32> {
    let mut offset = core.ip;
    let mut real_byte_count = 0;
    let mut bytes: Vec<f32> = Vec::new();
    for i in 0..bytecount {
        let byte = core.memory[offset + i as usize];
        if byte == i16::MIN {
            bytes.push(
                unpack_float(&[
                    core.memory[offset + 1 + i as usize],
                    core.memory[offset + 2 + i as usize],
                ])
                .expect("Couldn't convert bytes from i16 to float"),
            );
            offset += 2;
            real_byte_count += 3;
        } else if byte == i16::MAX {
            bytes.push(convert_reg_byte_to_command(
                core.memory[offset + 1 + i as usize],
                core,
            ));
            offset += 1;
            real_byte_count += 2;
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
        let byte = core.memory[core.ip + 1 + (i * 2) as usize];
        bytes.push(byte);
    }
    core.ip += (count * 2) as usize;
    bytes
}
#[derive(Debug)]
struct Core {
    ip: usize,
    stack: Vec<f32>,
    r1: i16,
    r2: i16,
    r3: i16,
    r4: i16,
    f1: f32,
    f2: f32,
    on: bool,
    memory: Vec<i16>,
}
impl Core {
    fn new() -> Core {
        Core {
            ip: 0,
            stack: Vec::new(),
            r1: 0,
            r2: 0,
            r3: 0,
            r4: 0,
            f1: 0.0,
            f2: 0.0,
            on: true,
            memory: Vec::new(),
        }
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
    NOP,
    IO,
    Loadf,
    Call,
    Return
}
fn convert_int_to_command(i: i16) -> CommandType {
    match i {
        0 => CommandType::Add,
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
        32 => CommandType::NOP,
        33 => CommandType::IO,
        34 => CommandType::Call,
        35=>CommandType::Return,
        _ => CommandType::NOP,
    }
}
fn pack_command(c: CommandType) -> i16 {
    match c {
        CommandType::Add => 0,
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
        CommandType::NOP => 32,
        CommandType::IO => 33,
        CommandType::Call=>34,
        CommandType::Return=>35,
        _ => 0,
    }
}
fn pack_register(r: CommandType) -> Vec<i16> {
    vec![
        i16::MAX,
        match r {
            CommandType::R1 => 1,
            CommandType::R2 => 2,
            CommandType::R3 => 3,
            CommandType::R4 => 4,
            CommandType::F1 => 5,
            CommandType::F2 => 6,
            CommandType::IP => 7,
            CommandType::SP => 8,
            _ => 0,
        },
    ]
}
fn convert_reg_byte_to_command(reg: i16, machine: &Core) -> f32 {
    match reg {
        1 => machine.r1 as f32,
        2 => machine.r2 as f32,
        3 => machine.r3 as f32,
        4 => machine.r4 as f32,
        5 => machine.f1,
        6 => machine.f2,
        7 => machine.ip as f32,
        8 => machine.stack.len() as f32,
        _ => -1.0,
    }
}
fn set_reg(reg: i16, machine: &mut Core, value: f32) {
    match reg {
        1 => machine.r1 = value as i16,
        2 => machine.r2 = value as i16,
        3 => machine.r3 = value as i16,
        4 => machine.r4 = value as i16,
        5 => machine.f1 = value,
        6 => machine.f2 = value,
        7 => machine.ip = value as usize,
        8 => machine.stack.resize(value as usize, 0.0),
        _ => (),
    }
}
