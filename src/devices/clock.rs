use crate::devices::RawDevice;
use crate::vm::Machine;
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Debug)]
pub struct Clock {}

impl Clock {
    pub fn new() -> Self {
        Clock {}
    }
    fn read(&self) -> f32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Couldn't get time")
            .as_secs_f32()
    }
}
pub fn driver(machine: &mut Machine, command: i16, device_id: usize) {
    match command {
        0 => {
            if let RawDevice::Clock(clock) = &machine.devices[device_id].contents {
                machine
                    .core
                    .stack
                    .push(clock.read() as f64, &mut machine.core.srp);
                if machine.debug {
                    println!("IO.clock.read");
                }
            } else {
                machine.core.stack.push(0.0, &mut machine.core.srp);
            }
        }
        _ => {}
    }
}
