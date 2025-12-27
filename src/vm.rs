use std::vec;
use prompted::input;
use crate::util::*;
use crate::devices::{Device,RawDevice};
use crate::devices::disk::{Disk};
pub enum Bytecode{
    Command(CommandType),
    Register(CommandType),
    Float(f32),
    Int(i16),
}
fn exec_bytecode(machine: &mut Machine){
    let byte = convert_int_to_command(take_bytes(&mut machine.core, 1)[0] as i16);
    let disk=if let RawDevice::Disk(disk)=&mut machine.devices[0].contents{
        Some(disk)
    }else{None}.expect("Could not get disk");
    if machine.debug{
        print!("%{:07}: ",machine.core.ip-1);
    }
    match byte {
        CommandType::Add => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = (args[0] + args[1]) as i16;
            if machine.debug{
                println!("Add {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Sub => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = (args[0] - args[1]) as i16;
            if machine.debug{
                println!("Sub {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Mul => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = (args[0] * args[1]) as i16;
            if machine.debug{
                println!("Mul {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Div => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = (args[0] / args[1]) as i16;
            if machine.debug{
                println!("Div {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Greater => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = (args[0] > args[1]) as i16;
            if machine.debug{
                println!("GreaterThan {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Addf => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.f1 = args[0] + args[1];
            if machine.debug{
                println!("Addf {} {} -> {}", args[0], args[1], machine.core.f1);
            }
        }
        CommandType::Subf => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.f1 = args[0] - args[1];
            if machine.debug{
                println!("Subf {} {} -> {}", args[0], args[1], machine.core.f1);
            }
        }
        CommandType::Mulf => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.f1 = args[0] * args[1];
            if machine.debug{
                println!("Mulf {} {} -> {}", args[0], args[1], machine.core.f1);
            }
        }
        CommandType::Divf => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.f1 = args[0] / args[1];
            if machine.debug{
                println!("Divf {} {} -> {}", args[0], args[1], machine.core.f1);
            }
        }
        CommandType::Mod => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = (args[0] % args[1]) as i16;
            if machine.debug{
                println!("Modulo {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Pop => {
            let val = machine.core.stack.pop(&mut machine.core.srp);
            let reg= take_registers(&mut machine.core, 1)[0];
            set_reg(reg, &mut machine.core, val);
            if machine.debug{
                println!("Pop {} -> R{}",val,reg);
            }
        }
        CommandType::LessThan => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = (args[0] < args[1]) as i16;
            if machine.debug{
                println!("LessThan {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Jump => {
            let addr = take_bytes(&mut machine.core, 1)[0];
            machine.core.ip = addr as usize;
            if machine.debug{
                println!("Jump {}",addr);
            }
        }
        CommandType::And => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = args[0] as i16 & args[1] as i16;
            if machine.debug{
                println!("And {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Or => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = args[0] as i16 | args[1] as i16;
            if machine.debug{
                println!("Or {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Not => {
            let args = take_bytes(&mut machine.core, 1);
            machine.core.r1 = !(args[0] as i16);
            if machine.debug{
                println!("Not {} -> {}", args[0], machine.core.r1);
            }
        }
        CommandType::Xor => {
            let args = take_bytes(&mut machine.core, 2);
            machine.core.r1 = args[0] as i16 ^ (args[1] as i16);
            if machine.debug{
                println!("Xor {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Push => {
            let args = take_bytes(&mut machine.core, 1);
            machine.core.stack.push(args[0], &mut machine.core.srp);
            if machine.debug{
                println!("Push {}",args[0]);
            }
        }
        CommandType::Mov => {
            let args = take_bytes(&mut machine.core, 1);
            let reg= take_registers(&mut machine.core, 1)[0];
            set_reg(reg, &mut machine.core, args[0]);
            if machine.debug{
                println!("Mov {} -> R{}",args[0],reg);
            }
        }
        CommandType::JumpNotZero => {
            let args = take_bytes(&mut machine.core, 2);
            if args[1] != 0.0 {
                machine.core.ip = args[0] as usize;
            }
            if machine.debug{
                println!("JumpNotZero {} {}",args[0],args[1]);
            }

        }
        CommandType::JumpZero=>{
            let args = take_bytes(&mut machine.core, 2);
            if args[1] == 0.0 {
                machine.core.ip = args[0] as usize;
            }
            if machine.debug{
                println!("JumpZero {} {}",args[0],args[1]);
            }
        }
        CommandType::Load => {
            let args = take_bytes(&mut machine.core, 1);
            let val = machine.core.memory[args[0] as usize] as f32;
            let reg= take_registers(&mut machine.core, 1)[0];
            set_reg(reg,&mut  machine.core, val);
            if machine.debug{
                println!("Load %{} -> R{}",args[0],reg);
            }
        }
        CommandType::Store => {
            let args = take_bytes(&mut machine.core, 2);
            if args[1].fract() == 0.0 {
                machine.core.memory[args[0] as usize] = args[1] as i16;}else{
                let f = convert_float(args[1]);
                machine.core
                    .memory
                    .splice(args[0] as usize..args[0] as usize + f.len(), f);
            }
            if machine.debug{
                println!("Store {} -> %{}",args[1],args[0]);
            }
        }
        CommandType::Exit => {
            machine.core.on = false;
            if machine.debug{
                println!("Exit");
            }
        }
        CommandType::Loadf => {
            let args = take_bytes(&mut machine.core, 1);
            let val = unpack_float(&machine.core.memory[args[0] as usize..args[0] as usize + 1usize])
                .expect(&format!("Couldn't get float at memory address {}", args[0]));
            let reg= take_registers(&mut machine.core, 1)[0];
            set_reg(reg, &mut machine.core, val);
            if machine.debug{
                println!("Loadf %{} -> R{}",args[0],reg);
            }
        }
        CommandType::IO => {
            //io(device,command), driverags are on stack
            let args = take_bytes(&mut machine.core, 2);
            if machine.debug{
                println!("IO {} {}",args[0],args[1]);
            }
            let driver=(machine.devices[args[0] as usize].driver.clone())(machine,args[1] as i16);
        }
        CommandType::Call=>{
            //call(fnptr,argcount,...args)
            let cargs = take_bytes(&mut machine.core, 2);
            let fnargs= take_bytes(&mut machine.core, cargs[1] as i16);
            machine.core.stack.push(machine.core.ip as f32, &mut machine.core.srp);
            for i in &fnargs{
                machine.core.stack.push(*i, &mut machine.core.srp);
            }
            machine.core.ip=cargs[0] as usize;
            if machine.debug{
                println!("Call {} Array[{},{:?}]",cargs[0],cargs[1],fnargs);
            }
        }
        CommandType::Return=>{
            //return(returned_byte_count)
            let args= take_bytes(&mut machine.core, 1);
            machine.core.ip= machine.core.stack.remove(machine.core.srp-(args[0] as usize), &mut machine.core.srp) as usize;
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
pub struct Machine{
    devices: Vec<Device>,
    core: Core,
    debug: bool,
    on: bool,
}
impl Machine {
    pub fn new(debug:bool) -> Machine {
        let m= Machine {
            devices:vec![Device {
                driver: |machine,command|{
                    let disk=if let RawDevice::Disk(disk)=&mut machine.devices[0].contents{
                        Some(disk)
                    }else{None}.expect("Could not get disk");
                    match command as i16 {
                        0 => {
                            //read(section,addr,len,dest)
                            let cargs = pop_stack(&mut machine.core, 4);
                            for i in (cargs[1] as usize)..(cargs[1] + cargs[2]) as usize{
                                if machine.core.memory.len()<=cargs[3] as usize+(i-cargs[1] as usize){
                                    machine.core.memory.resize(cargs[3] as usize+(i-cargs[1] as usize)+1,0);
                                }
                                machine.core.memory[cargs[3] as usize+(i-cargs[1] as usize)]=disk[cargs[0] as usize].data[i];
                            }
                            if machine.debug{
                                println!("IO.disk.read disk.%[{} {}] {} ->%{}",cargs[0],cargs[1],cargs[2],cargs[3]);
                            }
                        }
                        1 => {
                            //write(section,addr,byte)
                            let cargs = pop_stack(&mut machine.core, 3);
                            disk[cargs[0] as usize].data[cargs[1] as usize] = cargs[2] as i16;
                            if machine.debug{
                                println!("IO.disk.write {} -> disk.%[{} {}]",cargs[2],cargs[0],cargs[1]);
                            }
                        }
                        2 =>{
                            //loadSectors(start,count,dest)
                            let cargs = pop_stack(&mut machine.core, 3).iter().map(|i| *i as usize).collect::<Vec<usize>>();
                            let mut next_mem=cargs[2];
                            for i in cargs[0]..cargs[0]+cargs[1]{
                                for (j,byte) in disk[i].data.iter().enumerate(){
                                    if machine.core.memory.len()<=next_mem{
                                        machine.core.memory.resize(next_mem+1,0);
                                    }
                                    machine.core.memory[next_mem]=*byte;
                                    next_mem+=1;
                                }
                            }
                            if machine.debug{
                                println!("IO.disk.loadSectors disk.%[{}] {} ->%{}",cargs[0],cargs[1],cargs[2]);
                            }
                        }
                        _ => {}
                    }

                },
                contents: RawDevice::Disk(Vec::new()),
            }],
            core: Core::new(),
            debug,
            on: true,
        };
        m
    }
    pub fn run(&mut self){
        if let RawDevice::Disk(disk)=&mut self.devices[0].contents{
            if disk[0].data.len() >= 256 {
                self.core.memory = disk[0].data[0..256].to_vec();
            } else {
                self.core.memory = disk[0].data.clone();
            }
        }else{
            println!("No Disk Plugged In");
        }
        let mut debug_console=true;
        let mut breakpoints=Vec::new();
        while self.on {
            if self.debug&&debug_console || (breakpoints.contains(&self.core.ip)){
                if breakpoints.contains(&self.core.ip){
                    debug_console=true;
                }
                let input=input!("%{}>", self.core.ip);
                let command=input.split_whitespace().collect::<Vec<&str>>();
                if command.len()==0{
                    exec_bytecode(self)
                }else{
                    match command[0] {
                        "step"=>{ exec_bytecode(self)},
                        "dumpMem"=>{
                            let loc=0;
                            let mut len=self.core.memory.len()+1;
                            if len+loc>=self.core.memory.len(){
                                len=self.core.memory.len()-loc;
                            }
                            let data=&self.core.memory[loc..len+loc];
                            let mut printedData="".to_string();
                            for i in 0..(len as f32/50.0).ceil() as usize{
                                if i*50 <data.len(){
                                    let end=if data.len()>(i+1)*50{
                                        (i+1)*50
                                    }else{
                                        data.len()
                                    };
                                    printedData.extend(format!("%{:07}:{}\n",loc+i*50,
                                                               (&data[i*50..end]).iter().map(|x|format!(" {}",x)).collect::<String>()).chars())}
                            }
                            println!("{}",printedData)
                        },
                        "debugOff"=>{self.debug=false; println!("Debug Off"); exec_bytecode(self)},
                        "goto"=>{
                            let loc=command[1].parse::<usize>().expect("Could not parse goto loc");
                            self.core.ip=loc;
                        }
                        "stack"=>{
                            println!("{:?}",self.core.stack.data)
                        },
                        "exitConsole"=>{
                            debug_console=false;
                            exec_bytecode(self);
                        },
                        "breakpoint"=>{
                            breakpoints.push(command[1].parse::<usize>().expect("Could not parse breakpoint loc"));
                        }
                        "device"=>{
                            let device=command[1].parse::<usize>().expect("Invalid device ID");
                            println!("{:?}",self.devices[device].contents)
                        }
                        "registers"=>{
                            println!("R1: {}, R2: {}, R3: {}, R4: {}, F1: {}, F2: {}, SP: {}, SRP: {}, IP: {}",self.core.r1,self.core.r2,self.core.r3,self.core.r4,self.core.f1,self.core.f2,self.core.stack.len(),self.core.srp,self.core.ip)
                        },
                        "stop"=>{
                            self.on=false;
                        },
                        "nextCommand"=>{
                            println!("Command: {:?}",convert_int_to_command(self.core.memory[self.core.ip] as i16));
                        }
                        "readMem"=>{
                            let loc=command[1].parse::<usize>().expect("Invalid mem loc");
                            let mut len=command[2].parse::<usize>().expect("Invalid mem len")+1;
                            if len+loc>=self.core.memory.len(){
                                len=self.core.memory.len()-loc;
                            }
                            let data=&self.core.memory[loc..len+loc];
                            let mut printedData="".to_string();
                            for i in 0..(len as f32/50.0).ceil() as usize{
                                if i*50 <data.len(){
                                    let end=if data.len()>(i+1)*50{
                                        (i+1)*50
                                    }else{
                                        data.len()
                                    };
                                    printedData.extend(format!("%{:07}:{}\n",loc+i*50,
                                                               (&data[i*50..end]).iter().map(|x|format!(" {}",x)).collect::<String>()).chars())}
                            }
                            println!("{}",printedData)
                        }
                        _=>{}
                    }}
            }else{
                exec_bytecode(self)
            }
            self.on=self.core.on;
        }
    }
    pub fn set_disk(&mut self, disk: Disk){
        self.devices[0].contents=RawDevice::Disk(disk);
    }
}

#[derive(Debug)]
pub struct Core {
    pub ip: usize,
    pub stack: Stack,
    pub r1: i16,
    pub r2: i16,
    pub r3: i16,
    pub r4: i16,
    pub f1: f32,
    pub f2: f32,
    pub srp: usize,
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
pub struct Stack{
    data: Vec<f32>,
}
impl Stack {
    fn new() -> Stack {
        Stack {
            data: Vec::new(),
        }
    }
    pub fn len(&self) -> usize{
        self.data.len()
    }
    fn push(&mut self, x: f32,srp:&mut usize) {
        if *srp>=self.data.len(){
            self.data.resize(*srp,0.0);
        }
        self.data.insert(*srp,x);
        *srp+=1;
    }
    pub fn pop(&mut self, srp:&mut usize) -> f32 {
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
    pub fn resize(&mut self, size: usize,srp:&mut usize){
        if size<=self.data.len() {
            *srp=size;
        }
        self.data.resize(size,0.0);
    }
}

#[repr(u8)]
#[derive(Debug,Copy,Clone)]
pub enum CommandType {
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
    JumpZero,
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

