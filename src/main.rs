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
mod test;
use test::run_cases;
fn main() {
    run_cases();
    //println!("{:?}");
}
