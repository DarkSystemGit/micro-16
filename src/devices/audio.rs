use std::vec;
use std::vec::Vec;
use tinyaudio;
//4 square, 2 triangle, 2 noise, 2 sawtooth, 2 sample
struct AudioDevice{
    pub square_channels: [u16;4],
    pub triangle_channels: [u16;2],
    pub noise_channels: [u16;2],
    pub sawtooth_channels: [u16;2],
    pub sample_channels: [Vec<u16>;2],
    pub sample_rate: u32,
}
impl AudioDevice{
    pub fn new()->AudioDevice{
        AudioDevice{
            square_channels: [0;4],
            triangle_channels: [0;2],
            noise_channels: [0;2],
            sawtooth_channels: [0;2],
            sample_channels: [Vec::new(),Vec::new()],
            sample_rate: 32000,
        }
    }
    pub fn channel_count(&self)->usize{
        4+2+2+2+2
    }
    pub fn set_frequency(&mut self, channel: usize, frequency: u16){
        //channel 0-3: square, 4-5: triangle, 6-7: noise, 8-9: sawtooth, 10-11: sample
        if channel <4{
            self.square_channels[channel]=frequency;
        }else if channel <6{
            self.triangle_channels[channel-4]=frequency;
        }else if channel <8{
            self.noise_channels[channel-6]=frequency;
        }else if channel <10{
            self.sawtooth_channels[channel-8]=frequency;
        }
    }
    pub fn read_sample(&self){

    }
    fn gen_sample(&self,data: &mut [f32],formula: fn(clock: f32,rate: u32)->u16,out_channels:u32){
        /*let mut clock = 0f32;
        move |data: /* Type */| {
            for samples in data.chunks_mut(out_channels as usize) {
                clock = (clock + 1.0) % self.sample_rate as f32;
                let value =
                    formula(clock,self.sample_rate);
                for sample in samples {
                    *sample = value;
                }
            }
        }*/
    }
}