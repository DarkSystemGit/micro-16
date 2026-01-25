mod devices;
mod executable;
mod util;
mod vm;
use crate::devices::disk::{Disk};
use crate::executable::{Bytecode};
use crate::vm::{CommandType};

mod test;
use test::run_cases;
fn main() {
    run_cases();
    //println!("{:?}");
}
