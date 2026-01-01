use crate::Bytecode::{BlockLoc, Command, ConstantLoc, Float, FunctionRef, Int, Register};
use crate::CommandType;
use crate::CommandType::{
    Add, IO, Jump, JumpNotZero, JumpZero, Load, Mod, Mov, Pop, Push, R1, R2, R3, R4, Sub,
};
use crate::devices::disk::{Disk, DiskSection, DiskSectionType};
use crate::util::*;
use std::collections::HashMap;
pub struct Library {
    name: String,
    fns: Vec<Fn>,
    pub constants: Vec<Vec<i16>>,
}
impl Library {
    pub fn new(name: String) -> Library {
        Library {
            name,
            fns: vec![],
            constants: vec![],
        }
    }
    pub fn add_constant(&mut self, constant: Vec<i16>) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }
    pub fn add_fn(&mut self, mut func: Fn) -> usize {
        func.name = format!("{}::{}", self.name, func.name);
        func.blocks.iter_mut().for_each(|block| {
            for i in block.iter_mut() {
                if let Bytecode::FunctionRef(funcRef) = i {
                    *i = Bytecode::FunctionRef(
                        funcRef.replace("self::", &format!("{}::", self.name)),
                    );
                }
            }
        });
        self.fns.push(func);
        self.fns.len() - 1
    }
    pub fn link(&self, exe: &mut Executable) {
        let const_offset = exe.data_offsets.len() as i16;
        for constant in &self.constants {
            exe.add_constant(constant.clone());
        }
        for mut func in self.fns.clone() {
            func.blocks.iter_mut().for_each(|block| {
                for i in block.iter_mut() {
                    if let Bytecode::ConstantLoc(constant) = i {
                        *i = Bytecode::ConstantLoc(*constant + const_offset);
                    }
                }
            });
            exe.add_fn(func);
        }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Executable {
    data: Vec<i16>,
    data_offsets: Vec<i16>,
    fns: Vec<Fn>,
    loader: Vec<i16>,
    max_loader_len: i16,
}
#[derive(Debug, Clone)]
pub enum Bytecode {
    Command(CommandType),
    Register(CommandType),
    Float(f32),
    Int(i16),
    FunctionRef(String),
    ConstantLoc(i16),
    BlockLoc(i16),
}
//Bytecode Executable Structure
//-mem offset
//-base sector
//-bytecode len
//-bytecode sector count
//-data len
//-data sector count
//bytecode
//data
impl Executable {
    pub(crate) fn new() -> Executable {
        Executable {
            data: Vec::new(),
            data_offsets: Vec::new(),
            fns: Vec::new(),
            //loader loads from base sector to bytecode sector count, taking only bytecode len%i16::MAX for th final sector.
            //Then, it loads from bytecode sector count+1 to bytecode sector count+data_sector count, loading only data len%i16::MAX for the final sector
            //All of this is loaded at mem offset
            //Pseudocode
            //let next_mem=exec[0];
            //for i in exec[1]..exec[1]+exec[3]+exec[5]-1{
            //  if i==exec[1]+(exec[3]-1){
            //      let fcount=exec[2]%i16::MAX
            //      load(i,fcount,next_mem);
            //      next_mem+=fcount
            //  }else if i==exec[1]+exec[3]+exec[5]-1{
            //      let dfcount=exec[4]%i16::MAX;
            //      load(i,dfcount,next_mem);
            //      next_mem+=dfcount;
            //  }else{
            //      load(i,i16::MAX,next_mem)
            //      next_mem+=i16::MAX
            //  }
            //}

            //Assembly Pesudocode
            //Load exec[0]->r2 //next_mem
            //Load exec[1]->r3
            //Load exec[3]->r4
            //Add r3,r4->r1
            //Load exec[5]->r4
            //Add r1,r4->r1
            //Push r1
            //Sub r1,1->r1 //r1 is max range
            //Copy r3->r4 //r5 is counter

            //LCondition:
            //Sub r1,r4->r1
            //JumpZero loopEnd
            //Load exec[3]->r1
            //Add r3,r1->r1
            //Sub r1,r4->r1
            //JumpNotZero r1,DataEndConditon
            //Load exec[2]->r1
            //Mod r1,i16::Max->r1
            //IO.disk.read r4,0,r1,r2 //section,addr,len,dist
            //Add r2,r1->r2
            //inc r4
            //Jump LCondition

            //DataEndCondtion:
            //Pop r1
            //Push r1
            //Sub r1,1->r1
            //Sub r4,r1->r1
            //JumpNotZero r1,RegLoad
            //Load exec[4]->r1
            //Mod r1,i16::MAX
            //IO.disk.read r4,0,r1,r2
            //Add r2,r1->r2
            //inc r4
            //Jump LCondition

            //RegLoad:
            //IO.disk.read r4,0,i16::MAX,r2
            //Add r2,i16::MAX->r2
            //inc r4
            //Jump LCondition

            //loopEnd:
            //Jump 256
            loader: Self::default_loader(512, 6),
            max_loader_len: 512,
        }
    }
    fn default_loader(max_loader_len: i16, header_len: i16) -> Vec<i16> {
        let mut f = Fn::new("loader".to_string());
        f.add_block(
            vec![
                Command(Push),
                Int(0), //dest
                Command(Push),
                Int(1), //count
                Command(Push),
                Int(0), //start sector,
                Command(IO),
                Int(0),
                Int(2), //loadSectors,
                Command(Load),
                Int(max_loader_len + 3),
                Register(R1), //exec::bytecode_sector_count,
                Command(Load),
                Int(max_loader_len + 1),
                Register(R2), //exec::base_sector
                Command(Load),
                Int(max_loader_len + 5),
                Register(R3), //exec::data_sector_count
                Command(Add),
                Register(R1),
                Register(R3), //total sectors
                Command(Push),
                Int(0), //dest
                Command(Push),
                Register(R1), //count
                Command(Push),
                Register(R2), //start sector
                Command(IO),
                Int(0),
                Int(2), //loadSectors
                Command(Jump),
                Int(max_loader_len + header_len),
            ],
            true,
        );
        f.build(0, &HashMap::new(), 0, &vec![])
    }
    fn set_loader(&mut self, loader: Vec<i16>) {
        if loader.len() > self.max_loader_len as usize {
            println!("Oversized executable loader");
        }
        self.loader = loader;
    }
    pub(crate) fn add_constant(&mut self, constant: Vec<i16>) -> usize {
        let offset = self.data.len();
        self.data.extend(constant);
        self.data_offsets.push(offset as i16);
        self.data_offsets.len() - 1
    }
    pub(crate) fn add_fn(&mut self, mut data: Fn) -> usize {
        let id = self.fns.len();
        data.id = id;
        self.fns.push(data);
        0
    }
    pub(crate) fn build(mut self, mut offset: usize, disk: &mut Disk) {
        let mut bytecode: Vec<i16> = vec![];
        let mut fn_map: HashMap<String, usize> = HashMap::new();
        let header_len = 6;
        let insertion_jump_len = 2;
        //loader
        offset += self.max_loader_len as usize - 1;
        //headers
        offset += header_len + insertion_jump_len;
        let mut main_loc = 0;
        let data_sec = self
            .fns
            .iter_mut()
            .enumerate()
            .fold(offset + 1, |acc, (i, func)| {
                if func.name == "main" {
                    main_loc = acc;
                }
                fn_map.insert(func.name.clone(), acc);
                acc + func
                    .blocks
                    .iter()
                    .map(|b| func.get_block_len(&b))
                    .sum::<usize>()
                    + 2 //entrypoint jump
            }) as i16;
        //bytecode.push(as i16);
        for (i, func) in self.fns.iter_mut().enumerate() {
            bytecode.extend(func.build(fn_map[&func.name], &fn_map, data_sec, &self.data_offsets))
        }

        Self::insert_bytecode_into_disk(
            &self,
            disk,
            bytecode,
            offset,
            main_loc,
            header_len + insertion_jump_len,
        );
    }
    fn insert_bytecode_into_disk(
        &self,
        disk: &mut Disk,
        bytecode: Vec<i16>,
        mut offset: usize,
        entrypoint: usize,
        header_len: usize,
    ) {
        //(total exe code len/max sector data).ceil()
        let code_sectors = ((offset + bytecode.len()) as f32 / i16::MAX as f32).ceil() as usize;
        let data_sectors = (self.data.len() as f32 / i16::MAX as f32).ceil() as usize;
        //[mem offset,base sector,bytecode len,bytecode sector count, data len, data sector count]
        let headers = vec![
            offset - header_len,
            ((offset - header_len) as f32 / i16::MAX as f32).floor() as usize,
            bytecode.len() + 2,
            code_sectors,
            self.data.len(),
            data_sectors,
        ];
        let insertion_jump = vec![pack_command(CommandType::Jump), entrypoint as i16];
        let executable = flatten_vec(vec![
            headers.iter().map(|x| *x as i16).collect(),
            insertion_jump.clone(),
            bytecode,
        ]);
        //remove headers for these calculations
        offset -= header_len;
        let base_sector = (offset as f32 / i16::MAX as f32).floor() as usize;
        let bsector_offset = (offset as f32 % i16::MAX as f32) as usize;
        let data_sector_count = (self.data.len() as f32 / i16::MAX as f32).ceil() as usize;
        for i in base_sector..code_sectors {
            if i == base_sector {
                let insert_len = match executable.len() < i16::MAX as usize {
                    true => executable.len(),
                    false => i16::MAX as usize,
                };
                resize_vec(bsector_offset + insert_len, &mut disk[i].data, 0);
                disk[i]
                    .data
                    .splice(bsector_offset.., executable[0..insert_len].to_vec());
            } else {
                disk[i].section_type = match disk[base_sector].section_type {
                    DiskSectionType::Entrypoint => DiskSectionType::Code,
                    DiskSectionType::Libary => DiskSectionType::Libary,
                    _ => DiskSectionType::Code,
                };
                let sector_start = (i16::MAX as usize) * (i - base_sector);
                let sector_end = (i16::MAX as usize) * (i - base_sector + 1);
                disk[i].data = executable[sector_start..sector_end].to_vec();
            }
        }

        for i in code_sectors..code_sectors + data_sector_count {
            resize_vec(
                i + 1,
                disk,
                DiskSection {
                    section_type: DiskSectionType::Data,
                    id: -1,
                    data: vec![],
                },
            );
            let iteration = i - code_sectors;
            let data_start = iteration * i16::MAX as usize;
            let data_end = match self.data.len() < (iteration + 1) * i16::MAX as usize {
                false => (iteration + 1) * i16::MAX as usize,
                true => self.data.len(),
            };
            disk[i] = DiskSection {
                section_type: DiskSectionType::Data,
                id: i as i16,
                data: self.data[data_start..data_end].to_vec(),
            };
        }
        let mut loader = self.loader.clone();
        resize_vec(self.max_loader_len as usize, &mut loader, 0);
        disk[0]
            .data
            .splice(0..(self.max_loader_len - 1) as usize, loader);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Fn {
    name: String,
    blocks: Vec<Vec<Bytecode>>,
    entrypoint: usize,
    id: usize,
    loc: usize,
}
impl Fn {
    pub(crate) fn new(name: String) -> Fn {
        Fn {
            name,
            blocks: vec![],
            entrypoint: 0,
            id: 0,
            loc: 0,
        }
    }
    pub fn new_with_blocks(name: String, blocks: Vec<Vec<Bytecode>>) -> Fn {
        let mut f = Fn {
            name,
            blocks: vec![],
            entrypoint: 0,
            id: 0,
            loc: 0,
        };
        for block in blocks {
            f.add_block(block, false);
        }
        f
    }
    pub(crate) fn add_block(&mut self, block: Vec<Bytecode>, entrypoint: bool) -> usize {
        self.blocks.push(block);
        if entrypoint {
            self.entrypoint = self.blocks.len() - 1;
        }
        self.blocks.len() - 1
    }
    fn build(
        &mut self,
        pos: usize,
        fn_map: &HashMap<String, usize>,
        data_sec: i16,
        data_offsets: &Vec<i16>,
    ) -> Vec<i16> {
        let mut block_map: HashMap<usize, usize> = HashMap::new();
        let mut bytecode = Vec::new();
        self.blocks.iter().enumerate().fold(pos + 2, |acc, (i, b)| {
            block_map.insert(i, acc);
            acc + self.get_block_len(b)
        });
        bytecode.extend_from_slice(&[19, block_map[&(self.entrypoint)] as i16]);
        for (i, block) in self.blocks.iter_mut().enumerate() {
            let blockCode = flatten_vec(
                block
                    .iter()
                    .map(|inst| match inst {
                        Command(c) => vec![pack_command(*c)],
                        Register(r) => pack_register(*r),
                        Float(f) => pack_float(*f),
                        Int(i) => vec![*i],
                        FunctionRef(f) => vec![fn_map[f] as i16],
                        ConstantLoc(c) => vec![data_sec + data_offsets[*c as usize]],
                        BlockLoc(b) => vec![block_map[&(*b as usize)] as i16],
                    })
                    .collect::<Vec<Vec<i16>>>(),
            );
            bytecode.extend(blockCode.iter().map(|x| *x));
        }
        bytecode
    }
    fn get_block_len(&self, block: &Vec<Bytecode>) -> usize {
        block
            .iter()
            .map(|inst| match inst {
                Command(_c) => 1,
                Register(_r) => 3,
                Float(_f) => 4,
                Int(_i) => 1,
                FunctionRef(_f) => 1,
                ConstantLoc(_c) => 1,
                BlockLoc(_b) => 1,
            })
            .collect::<Vec<usize>>()
            .iter()
            .sum()
    }
}
