use crate::Bytecode;
use crate::devices;
use crate::devices::disk::Disk;
use crate::devices::{Device, RawDevice};
use crate::util::*;
use prompted::input;
use std::ops::Range;
use std::panic;
fn exec_bytecode(machine: &mut Machine) {
    let byte = convert_int_to_command(take_bytes(machine, 1)[0] as i16);
    if machine.debug {
        print!("%{:07}: ", machine.core.ip - 1);
    }
    match byte {
        CommandType::Add => {
            //add(i16,i16) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = (args[0] + args[1]) as i16;
            if machine.debug {
                println!("Add {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Sub => {
            //sub(i16,i16) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = (args[0] - args[1]) as i16;
            if machine.debug {
                println!("Sub {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Mul => {
            //mul(i16,i16) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = (args[0] * args[1]) as i16;
            if machine.debug {
                println!("Mul {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Div => {
            //div(i16,i16) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = (args[0] / args[1]) as i16;
            if machine.debug {
                println!("Div {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Greater => {
            //greater(i16,i16) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = (args[0] > args[1]) as i16;
            if machine.debug {
                println!("GreaterThan {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Addf => {
            //addf(f32,f32) -> f1
            let args = take_bytes(machine, 2);
            machine.core.f1 = args[0] + args[1];
            if machine.debug {
                println!("Addf {} {} -> {}", args[0], args[1], machine.core.f1);
            }
        }
        CommandType::Subf => {
            //subf(f32,f32) -> f1
            let args = take_bytes(machine, 2);
            machine.core.f1 = args[0] - args[1];
            if machine.debug {
                println!("Subf {} {} -> {}", args[0], args[1], machine.core.f1);
            }
        }
        CommandType::Mulf => {
            //mulf(f32,f32) -> f1
            let args = take_bytes(machine, 2);
            machine.core.f1 = args[0] * args[1];
            if machine.debug {
                println!("Mulf {} {} -> {}", args[0], args[1], machine.core.f1);
            }
        }
        CommandType::Divf => {
            //divf(f32,f32) -> f1
            let args = take_bytes(machine, 2);
            machine.core.f1 = args[0] / args[1];
            if machine.debug {
                println!("Divf {} {} -> {}", args[0], args[1], machine.core.f1);
            }
        }
        CommandType::Mod => {
            //mod(f32,f32) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = (args[0] % args[1]) as i16;
            if machine.debug {
                println!("Modulo {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Pop => {
            //pop() -> Register
            let val = machine.core.stack.pop(&mut machine.core.srp);
            let reg = take_registers(machine, 1)[0];
            set_reg(reg, &mut machine.core, val);
            if machine.debug {
                println!("Pop {} -> R{}", val, reg);
            }
        }
        CommandType::LessThan => {
            //less_than(f32,f32) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = (args[0] < args[1]) as i16;
            if machine.debug {
                println!("LessThan {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Jump => {
            //jump(address)
            let addr = take_bytes(machine, 1)[0];
            machine.core.ip = addr as usize;
            if machine.debug {
                println!("Jump {}", addr);
            }
        }
        CommandType::And => {
            //and(i16,i16) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = args[0] as i16 & args[1] as i16;
            if machine.debug {
                println!("And {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Or => {
            //or(i16,i16) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = args[0] as i16 | args[1] as i16;
            if machine.debug {
                println!("Or {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Not => {
            //not(i16) -> r1
            let args = take_bytes(machine, 1);
            machine.core.r1 = !(args[0] as i16);
            if machine.debug {
                println!("Not {} -> {}", args[0], machine.core.r1);
            }
        }
        CommandType::Xor => {
            //xor(i16,i16) -> r1
            let args = take_bytes(machine, 2);
            machine.core.r1 = args[0] as i16 ^ (args[1] as i16);
            if machine.debug {
                println!("Xor {} {} -> {}", args[0], args[1], machine.core.r1);
            }
        }
        CommandType::Push => {
            //push(f32)
            let args = take_bytes(machine, 1);
            machine.core.stack.push(args[0], &mut machine.core.srp);
            if machine.debug {
                println!("Push {}", args[0]);
            }
        }
        CommandType::Mov => {
            //mov(f32) -> Register
            let args = take_bytes(machine, 1);
            let reg = take_registers(machine, 1)[0];
            set_reg(reg, &mut machine.core, args[0]);
            if machine.debug {
                println!("Mov {} -> R{}", args[0], reg);
            }
        }
        CommandType::JumpNotZero => {
            //jnz(address,f32)
            let args = take_bytes(machine, 2);
            if args[1] != 0.0 {
                machine.core.ip = args[0] as usize;
            }
            if machine.debug {
                println!("JumpNotZero {} {}", args[0], args[1]);
            }
        }
        CommandType::JumpZero => {
            //jz(address,f32)
            let args = take_bytes(machine, 2);
            if args[1] == 0.0 {
                machine.core.ip = args[0] as usize;
            }
            if machine.debug {
                println!("JumpZero {} {}", args[0], args[1]);
            }
        }
        CommandType::Load => {
            //load(address) -> Register
            let args = take_bytes(machine, 1);
            let val = machine.memory.read(args[0] as usize, machine) as f32;
            let reg = take_registers(machine, 1)[0];
            set_reg(reg, &mut machine.core, val);
            if machine.debug {
                println!("Load %{} -> R{}", args[0], reg);
            }
        }
        CommandType::Store => {
            //store(address,f32)
            let args = take_bytes(machine, 2);
            if args[1].fract() == 0.0 {
                machine
                    .memory
                    .write(args[0] as usize, args[1] as i16, &mut machine.core);
            } else {
                let f = convert_float(args[1]);
                machine.memory.write_range(
                    args[0] as usize..args[0] as usize + f.len(),
                    f,
                    &mut machine.core,
                );
            }
            if machine.debug {
                println!("Store {} -> %{}", args[1], args[0]);
            }
        }
        CommandType::Exit => {
            //exit()
            machine.core.on = false;
            if machine.debug {
                println!("Exit");
            }
        }
        CommandType::Loadf => {
            //loadf(address) -> Register
            let args = take_bytes(machine, 1);
            let val_bytes = &machine
                .memory
                .read_range(args[0] as usize..args[0] as usize + 2usize, machine);
            let val = unpack_float(val_bytes)
                .expect(&format!("Couldn't get float at memory address {}", args[0]));
            let reg = take_registers(machine, 1)[0];
            set_reg(reg, &mut machine.core, val);
            if machine.debug {
                println!("Loadf %{} -> R{}", args[0], reg);
            }
        }
        CommandType::IO => {
            //io(device,command), driverags are on stack
            let args = take_bytes(machine, 2);
            if machine.debug {
                println!("IO {} {}", args[0], args[1]);
            }
            (machine.devices[args[0] as usize].driver.clone())(
                machine,
                args[1] as i16,
                args[0] as usize,
            );
        }
        CommandType::Call => {
            //call(fnptr,argcount,...args)
            let cargs = take_bytes(machine, 2);
            let fnargs = take_bytes(machine, cargs[1] as i16);
            machine
                .core
                .stack
                .push(machine.core.ip as f32, &mut machine.core.srp);
            for i in &fnargs {
                machine.core.stack.push(*i, &mut machine.core.srp);
            }
            machine.core.ip = cargs[0] as usize;
            if machine.debug {
                println!("Call %{} Array[len: {},{:?}]", cargs[0], cargs[1], fnargs);
            }
        }
        CommandType::Return => {
            //return(returned_byte_count)
            let args = take_bytes(machine, 1);
            machine.core.ip = machine.core.stack.remove(
                machine.core.srp - (args[0] as usize + 1),
                &mut machine.core.srp,
            ) as usize;
            if machine.debug {
                println!("Return {}", args[0]);
            }
        }
        CommandType::NOP => {
            //nop()
            if machine.debug {
                println!("NOP");
            }
        }
        _ => {}
    }
}

fn take_bytes(machine: &mut Machine, bytecount: i16) -> Vec<f32> {
    let mut offset = machine.core.ip;
    let mut real_byte_count = 0;
    let mut bytes: Vec<f32> = Vec::new();
    for i in 0..bytecount {
        let byte = machine.memory.read(offset + i as usize, machine);
        if byte == i16::MIN {
            match machine.memory.read(offset + i as usize + 1, machine) {
                0 => {
                    bytes.push(
                        unpack_float(&[
                            machine.memory.read(offset + 2 + i as usize, machine),
                            machine.memory.read(offset + 3 + i as usize, machine),
                        ])
                        .expect("Couldn't convert bytes from i16 to float"),
                    );
                    offset += 3;
                    real_byte_count += 4;
                }
                1 => {
                    bytes.push(convert_reg_byte_to_command(
                        machine.memory.read(offset + 2 + i as usize, machine),
                        &machine.core,
                    ));
                    offset += 2;
                    real_byte_count += 3;
                }
                _ => {}
            }
        } else {
            bytes.push(byte as f32);
            real_byte_count += 1;
        }
    }
    machine.core.ip += real_byte_count;
    bytes
}
fn take_registers(machine: &mut Machine, count: i16) -> Vec<i16> {
    let mut bytes: Vec<i16> = Vec::new();
    for i in 0..count {
        let byte = machine
            .memory
            .read(machine.core.ip + 2 + (i * 3) as usize, machine);
        bytes.push(byte);
    }
    machine.core.ip += (count * 3) as usize;
    bytes
}
pub struct Machine {
    pub devices: Vec<Device>,
    pub core: Core,
    pub debug: bool,
    pub memory: Memory,
    on: bool,
}
impl Machine {
    pub fn new(debug: bool) -> Machine {
        let m = Machine {
            devices: devices::get_device_list(),
            core: Core::new(),
            debug,
            on: true,
            memory: Memory::new(4 * 1024 * 1024), //4MB max
        };
        m
    }
    fn panic(&self, addr: usize) {
        println!("PANIC at %{}", addr);
        println!("__________________________________________");
        println!("State:");
        println!("Core:");
        println!("IP: {}", self.core.ip);
        println!("Registers:");
        println!("R1: {}", self.core.r1);
        println!("R2: {}", self.core.r2);
        println!("R3: {}", self.core.r3);
        println!("R4: {}", self.core.r4);
        println!("F1: {}", self.core.f1);
        println!("F2: {}", self.core.f2);
        println!("Stack:");
        println!("SRP: {}", self.core.srp);
        println!("Stack Pointer: {}", self.core.stack.len());
        println!("Stack Contents:");
        for i in 0..self.core.stack.len() {
            println!("%{}: {}", i, self.core.stack.data[i]);
        }
        println!("Memory");
        println!("{:?}", self.memory.data);
    }
    pub fn run(&mut self) {
        if let RawDevice::Disk(disk) = &mut self.devices[0].contents {
            if disk[0].data.len() >= 256 {
                self.memory
                    .write_range(0..256, disk[0].data[0..256].to_vec(), &mut self.core);
            } else {
                self.memory.write_range(
                    0..disk[0].data.len(),
                    disk[0].data.clone(),
                    &mut self.core,
                );
            }
        } else {
            println!("No Disk Plugged In");
        }
        let mut debug_console = true;
        let mut breakpoints = Vec::new();
        while self.on {
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                if self.debug && debug_console || (breakpoints.contains(&self.core.ip)) {
                    if breakpoints.contains(&self.core.ip) {
                        debug_console = true;
                    }
                    let input = input!("%{}>", self.core.ip);
                    let command = input.split_whitespace().collect::<Vec<&str>>();
                    if command.len() == 0 {
                        exec_bytecode(self)
                    } else {
                        match command[0] {
                            "help" => {
                                println!("Available commands:");
                                println!("  help - Display this help message");
                                println!("  step - Execute the next instruction");
                                println!("  dumpMem - Dump memory contents");
                                println!("  debugOff - Exit Debugger");
                                println!("  goto - Jump to an address");
                                println!("  stack - Display the stack");
                                println!("  exitConsole - Exit debug console");
                                println!("  breakpoint - Set a breakpoint");
                                println!("  device - Dump a device");
                                println!("  registers - Dump registers");
                                println!("  stop - Stops execution");
                                println!(
                                    "  nextCommand - Reads the byte at IP and displays it as a command"
                                );
                                println!(
                                    "  readMem - Reads x bytes from an address and displays it"
                                );
                            }
                            "step" => exec_bytecode(self),
                            "dumpMem" => {
                                let loc = 0;
                                let mut len = self.memory.len() + 1;
                                if len + loc >= self.memory.len() {
                                    len = self.memory.len() - loc;
                                }
                                let data = &self.memory.read_range(loc..len + loc, self);
                                let mut printed_data = "".to_string();
                                for i in 0..(len as f32 / 50.0).ceil() as usize {
                                    if i * 50 < data.len() {
                                        let end = if data.len() > (i + 1) * 50 {
                                            (i + 1) * 50
                                        } else {
                                            data.len()
                                        };
                                        printed_data.extend(
                                            format!(
                                                "%{:07}:{}\n",
                                                loc + i * 50,
                                                (&data[i * 50..end])
                                                    .iter()
                                                    .map(|x| format!(" {}", x))
                                                    .collect::<String>()
                                            )
                                            .chars(),
                                        )
                                    }
                                }
                                println!("{}", printed_data)
                            }
                            "debugOff" => {
                                self.debug = false;
                                println!("Debug Off");
                                exec_bytecode(self)
                            }
                            "goto" => {
                                let loc = command[1]
                                    .parse::<usize>()
                                    .expect("Could not parse goto loc");
                                self.core.ip = loc;
                            }
                            "stack" => {
                                println!("{:?}", self.core.stack.data)
                            }
                            "exitConsole" => {
                                debug_console = false;
                                exec_bytecode(self);
                            }
                            "breakpoint" => {
                                breakpoints.push(
                                    command[1]
                                        .parse::<usize>()
                                        .expect("Could not parse breakpoint loc"),
                                );
                            }
                            "device" => {
                                let device =
                                    command[1].parse::<usize>().expect("Invalid device ID");
                                println!("{:?}", self.devices[device].contents)
                            }
                            "registers" => {
                                println!(
                                    "R1: {}, R2: {}, R3: {}, R4: {}, F1: {}, F2: {}, SP: {}, SRP: {}, IP: {}",
                                    self.core.r1,
                                    self.core.r2,
                                    self.core.r3,
                                    self.core.r4,
                                    self.core.f1,
                                    self.core.f2,
                                    self.core.stack.len(),
                                    self.core.srp,
                                    self.core.ip
                                )
                            }
                            "stop" => {
                                self.on = false;
                            }
                            "nextCommand" => {
                                println!(
                                    "Command: {:?}",
                                    convert_int_to_command(
                                        self.memory.read(self.core.ip, &self) as i16
                                    )
                                );
                            }
                            "readMem" => {
                                let loc = command[1].parse::<usize>().expect("Invalid mem loc");
                                let mut len =
                                    command[2].parse::<usize>().expect("Invalid mem len") + 1;
                                if len + loc >= self.memory.len() {
                                    len = self.memory.len() - loc;
                                }
                                let data = &self.memory.read_range(loc..len + loc, self);
                                let mut printed_data = "".to_string();
                                for i in 0..(len as f32 / 50.0).ceil() as usize {
                                    if i * 50 < data.len() {
                                        let end = if data.len() > (i + 1) * 50 {
                                            (i + 1) * 50
                                        } else {
                                            data.len()
                                        };
                                        printed_data.extend(
                                            format!(
                                                "%{:07}:{}\n",
                                                loc + i * 50,
                                                (&data[i * 50..end])
                                                    .iter()
                                                    .map(|x| format!(" {}", x))
                                                    .collect::<String>()
                                            )
                                            .chars(),
                                        )
                                    }
                                }
                                println!("{}", printed_data)
                            }
                            _ => {}
                        }
                    }
                } else {
                    exec_bytecode(self)
                }
            }));
            if result.is_err() {
                self.panic(self.core.ip);
                return;
            }
            self.on = self.core.on;
        }
    }
    pub fn set_disk(&mut self, disk: Disk) {
        self.devices[0].contents = RawDevice::Disk(disk);
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
            srp: 2,
            on: true,
        }
    }
}
#[derive(Debug)]
pub struct Memory {
    data: Vec<i16>,
    max_size: usize,
}
impl Memory {
    fn new(max_size: usize) -> Memory {
        Memory {
            data: Vec::new(),
            max_size,
        }
    }
    pub fn read(&self, index: usize, machine: &Machine) -> i16 {
        if index >= self.max_size {
            *machine
                .core
                .stack
                .data
                .get(index - self.max_size)
                .expect(&format!(
                    "Invalid Memory Access: Address %{} is out of bounds",
                    index
                )) as i16
        } else {
            self.data.get(index).copied().unwrap_or(0)
        }
    }
    pub fn read_range(&self, range: Range<usize>, machine: &Machine) -> Vec<i16> {
        if range.end > self.max_size {
            if (range.end - self.max_size) >= machine.core.stack.data.len() {
                panic!(
                    "Invalid Memory Access: Address %{} is out of bounds",
                    machine.core.stack.data.len()
                );
            }
            let mem_data = self.data[range.start..(self.max_size)].to_vec();
            let stackrange = flatten_vec(
                machine.core.stack.data[0..(range.end - self.max_size)]
                    .to_vec()
                    .iter()
                    .map(|x| convert_float_or_int_to_bytes(*x))
                    .collect(),
            );
            [mem_data, stackrange].concat();
        }
        self.data[range].to_vec()
    }
    pub fn write(&mut self, index: usize, value: i16, core: &mut Core) {
        if index >= self.max_size {
            *core
                .stack
                .data
                .get_mut(index - self.max_size)
                .expect(&format!(
                    "Invalid Memory Access: Address %{} is out of bounds",
                    index
                )) = value as f32;
        } else {
            if index < self.data.len() {
                self.data[index] = value;
            } else {
                self.data.resize(index + 1, 0);
                self.data[index] = value;
            }
        }
    }
    pub fn write_range(&mut self, range: Range<usize>, value: Vec<i16>, core: &mut Core) {
        if range.end > self.max_size {
            if range.end - self.max_size > core.stack.len() {
                panic!(
                    "Invalid Memory Access: Address %{} is out of bounds",
                    self.max_size + core.stack.len()
                );
            }
            self.data[range.start..self.max_size]
                .copy_from_slice(&value[0..self.max_size - range.start]);
            core.stack.data[0..(range.end - self.max_size)].copy_from_slice(
                value[self.max_size - range.start..]
                    .to_vec()
                    .iter()
                    .map(|x| *x as f32)
                    .collect::<Vec<f32>>()
                    .as_slice(),
            );
        }
        if range.end > self.data.len() {
            self.data.resize(range.end, 0);
        }
        self.data[range].copy_from_slice(&value);
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
}
#[derive(Debug)]
pub struct Stack {
    data: Vec<f32>,
}
impl Stack {
    fn new() -> Stack {
        Stack { data: Vec::new() }
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn push(&mut self, x: f32, srp: &mut usize) {
        if *srp >= self.data.len() {
            self.data.resize(*srp, 0.0);
        }
        self.data.insert(*srp, x);
        *srp += 1;
    }
    pub fn pop(&mut self, srp: &mut usize) -> f32 {
        self.remove(*srp - 1, srp)
    }

    pub fn remove(&mut self, index: usize, srp: &mut usize) -> f32 {
        *srp -= 1;
        self.data.remove(index)
    }
    pub fn resize(&mut self, size: usize, srp: &mut usize) {
        if size <= self.data.len() {
            *srp = size;
        }
        self.data.resize(size, 0.0);
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
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
    Return,
}
