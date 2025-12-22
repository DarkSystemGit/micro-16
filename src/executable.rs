use std::collections::HashMap;
use crate::{Bytecode, CommandType, Disk, DiskSection, DiskSectionType};
use crate::util::*;

#[derive(Debug,Clone)]
pub(crate) struct Executable {
    data: Vec<i16>,
    data_offsets: Vec<i16>,
    fns: Vec<Fn>,
    loader: Vec<i16>
}
impl Executable {
    pub(crate) fn new() -> Executable {
        Executable {
            data: Vec::new(),
            data_offsets: Vec::new(),
            fns: Vec::new(),
            loader: flatten_vec(vec![
                gen_io_read(256,0,6),


            ])
        }
    }
    fn set_loader(&mut self,loader: Vec<i16>) {
        if loader.len()>256{
            println!("Oversized executable loader");
        }
        self.loader=loader;
    }
    pub(crate) fn add_constant(&mut self, constant: Vec<i16>) ->usize{
        let offset=self.data.len();
        self.data.extend(constant);
        self.data_offsets.push(offset as i16);
        offset
    }
    pub(crate) fn add_fn(&mut self, mut data: Fn) ->usize{
        let id=self.fns.len();
        data.id=id;
        self.fns.push(data);
        0
    }
    pub(crate) fn build(mut self, mut offset: usize, disk: &mut Disk) {
        //Executable Structure
        //-mem offset
        //-base sector
        //-bytecode len
        //-bytecode sector count
        //-data len
        //-data sector count
        //bytecode
        //data
        let mut bytecode: Vec<i16>=vec![];
        let mut fn_map: HashMap<usize,usize>=HashMap::new();
        let header_len=6;
        let insertion_jump_len=2;
        //loader
        offset+=255;
        //headers
        offset+=header_len+insertion_jump_len;
        let mut main_loc =0;
        let data_sec=offset as i16+self.fns.iter_mut().enumerate().fold(
            offset,
            |acc,(i,func)|{
                if func.name=="main"{
                    main_loc =acc;
                }
                fn_map.insert(i, acc);
                acc+func.blocks.iter().map(|b|b.len()-1).sum::<usize>()
            }
        ) as i16;
        //bytecode.push(as i16);
        for (i,func) in self.fns.iter_mut().enumerate(){
            let mut block_map: HashMap<usize,usize>=HashMap::new();
            func.blocks.iter().enumerate().fold(
                fn_map[&i],
                |acc,(i,b)| {
                    block_map.insert(i, acc);
                    acc+b.len()
                }
            );

            for (i,block) in func.blocks.iter_mut().enumerate(){
                let jumps=&func.jumps[i];
                let constant_usage =&func.constant_accesses[i];
                jumps.iter().for_each(|j|{
                    let jump_loc =block[*j + 1] as usize;
                    block[j+1]=match convert_int_to_command(block[*j]){
                        CommandType::Jump=>{
                            block_map[&(jump_loc)]
                        },
                        CommandType::JumpNotZero=>{
                            block_map[&(jump_loc)]
                        },
                        CommandType::Call=>{
                            fn_map[&jump_loc]
                        },
                        _=>0
                    } as i16;
                });
                constant_usage.iter().for_each(|constant_loc|{
                    let constant=block[*constant_loc];
                    block[*constant_loc]= data_sec + self.data_offsets[constant as usize];
                });
                bytecode.extend(block.iter().map(|x|*x));
            }
        }
        Self::insert_bytecode_into_disk(&self, disk, bytecode, offset, main_loc, header_len+insertion_jump_len);
    }
    fn insert_bytecode_into_disk(&self,disk: &mut Disk,bytecode: Vec<i16>,mut offset:usize,entrypoint:usize,header_len:usize) {


        //(total exe code len/max sector data).ceil()
        let code_sectors=((offset+bytecode.len()) as f32/i16::MAX as f32).ceil() as usize;
        let data_sectors=(self.data.len() as f32/i16::MAX as f32).ceil() as usize;
        let headers=vec![((offset-header_len) as f32/i16::MAX as f32).floor() as usize,offset-header_len,bytecode.len()+2,code_sectors,self.data.len(),data_sectors];
        let insertion_jump=vec![pack_command(CommandType::Jump),entrypoint as i16];
        let executable= flatten_vec(vec![headers.iter().map(|x|*x as i16).collect(), insertion_jump.clone(), bytecode]);
        //remove headers for these calculations
        offset-=header_len;
        let base_sector=(offset as f32/i16::MAX as f32).floor() as usize;
        let bsector_offset=(offset as f32%i16::MAX as f32) as usize;
        let data_sector_count=(self.data.len() as f32/i16::MAX as f32).ceil() as usize;
        for i in base_sector..code_sectors{
            if i==base_sector{
                let insert_len=match executable.len()<i16::MAX as usize{
                    true=>executable.len(),
                    false=> i16::MAX as usize
                };
                resize_vec(bsector_offset+insert_len,&mut disk[i].data,0);
                disk[i].data.splice(bsector_offset..,executable[0..insert_len].to_vec());
            }else{
                disk[i].section_type =match disk[base_sector].section_type {
                    DiskSectionType::Entrypoint=>DiskSectionType::Code,
                    DiskSectionType::Libary=>DiskSectionType::Libary,
                    _=>DiskSectionType::Code,
                };
                let sector_start=(i16::MAX as usize)*(i-base_sector);
                let sector_end=(i16::MAX as usize)*(i-base_sector+1);
                disk[i].data=executable[sector_start..sector_end].to_vec();
            }
        }

        for i in code_sectors..code_sectors+data_sector_count{
            resize_vec(i+1,disk,DiskSection{
                section_type: DiskSectionType::Data,
                id:-1,
                data:vec![],
            });
            let iteration=i-code_sectors;
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
        //dbg!(disk);
    }

}

#[derive(Debug,Clone)]
pub(crate) struct Fn{
    name: String,
    blocks: Vec<Vec<i16>>,
    jumps: Vec<Vec<usize>>,
    constant_accesses: Vec<Vec<usize>>,
    id: usize,
    loc: usize,
}
impl Fn {
    pub(crate) fn new(name: String) -> Fn {
        Fn{
            name,
            blocks: vec![vec![19,0]],
            jumps: vec![vec![0]],
            constant_accesses: vec![vec![]],
            id:0,
            loc: 0
        }
    }
    pub(crate) fn add_block(&mut self, block: Vec<Bytecode>, entrypoint:bool) ->usize{
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
        self.constant_accesses.push(constant_usages);
        self.blocks[0][1]=(self.blocks.len()-1) as i16;
        self.blocks.len()-1
    }
}