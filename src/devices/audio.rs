use crate::util::flatten_vec;
use arc_swap::{ArcSwap, ArcSwapAny, Guard};
use std::{sync::Arc, time::Instant, vec};
use tinyaudio::prelude::*;
//4 square, 2 triangle, 2 sawtooth, 2 sample
struct AudioDevice {
    pub channels: ChannelCollection,
    pub sample_rate: u32,
    pub on: bool,
}
impl AudioDevice {
    pub fn new() -> AudioDevice {
        AudioDevice {
            channels: mutex_channels(flatten_vec(vec![
                gen_uninitialized_channels(gen_square_wave as WaveGenerator, 4),
                gen_uninitialized_channels(gen_triangle_wave as WaveGenerator, 2),
                gen_uninitialized_channels(gen_sawtooth_wave as WaveGenerator, 2),
                vec![
                    Channel::new_with_sample(vec![0.0], 0.5, [0.5, 0.5]),
                    Channel::new_with_sample(vec![0.0], 0.5, [0.5, 0.5]),
                ],
            ])),
            sample_rate: 32000,
            on: false,
        }
    }
    pub fn update_channel(&self, id: usize, update: ChannelUpdate) {
        modify_channel_collection_item(id, &(self.channels), update);
    }
    pub fn read_channel(&self, id: usize) -> Guard<Arc<Channel>> {
        get_channel_collection_item(id, &(self.channels))
    }
    pub fn run(&mut self) {
        self.on = true;
        let params = OutputDeviceParameters {
            channels_count: 2,
            sample_rate: 32000,
            channel_sample_count: 3200,
        };
        let _device = run_output_device(
            params,
            gen_wave(params.sample_rate as u32, Arc::clone(&(self.channels))),
        )
        .expect("Could not initialize audio device");
    }
}
fn gen_uninitialized_channels(wave: WaveGenerator, count: i32) -> Vec<Channel> {
    let mut r = vec![];
    for i in 0..count {
        r.push(Channel::new(0.0, 1.0, [0.5, 0.5], wave));
    }
    r
}
type ChannelCollection = Arc<Vec<ArcSwapAny<Arc<Channel>>>>;
fn mutex_channels(channels: Vec<Channel>) -> ChannelCollection {
    Arc::new(
        channels
            .iter()
            .map(|c| ArcSwap::from(Arc::new(c.clone())))
            .collect(),
    )
}
fn get_channel_collection_item(id: usize, channels: &ChannelCollection) -> Guard<Arc<Channel>> {
    channels
        .get(id)
        .map(|c| c.load())
        .expect("Couldn't find channel")
}
fn modify_channel_collection_item(id: usize, channels: &ChannelCollection, update: ChannelUpdate) {
    if let Some(channel) = channels.get(id) {
        let uc = channel.load();
        if uc.freq != None {
            match update {
                ChannelUpdate::Volume(volume) => {
                    channel.store(Arc::new(Channel::new(
                        uc.freq.unwrap(),
                        volume,
                        uc.pan,
                        uc.wave.unwrap(),
                    )));
                }
                ChannelUpdate::Pan(pan) => {
                    channel.store(Arc::new(Channel::new(
                        uc.freq.unwrap(),
                        uc.volume,
                        pan,
                        uc.wave.unwrap(),
                    )));
                }
                ChannelUpdate::Frequency(freq) => {
                    channel.store(Arc::new(Channel::new(
                        freq,
                        uc.volume,
                        uc.pan,
                        uc.wave.unwrap(),
                    )));
                }
                _ => {}
            }
        } else {
            match update {
                ChannelUpdate::Volume(volume) => {
                    channel.store(Arc::new(Channel::new_with_sample(
                        uc.wave_sample.clone().unwrap(),
                        volume,
                        uc.pan,
                    )));
                }
                ChannelUpdate::Pan(pan) => {
                    channel.store(Arc::new(Channel::new_with_sample(
                        uc.wave_sample.clone().unwrap(),
                        uc.volume,
                        pan,
                    )));
                }
                ChannelUpdate::WaveSample(sample) => {
                    channel.store(Arc::new(Channel::new_with_sample(
                        sample, uc.volume, uc.pan,
                    )));
                }
                _ => {}
            }
        }
    }
}
enum ChannelUpdate {
    Volume(f32),
    Pan([f32; 2]),
    Frequency(f32),
    WaveSample(Vec<f32>),
}
type WaveGenerator = fn(f32, f32, f32) -> f32;
fn gen_wave(sample_rate: u32, channels: ChannelCollection) -> impl FnMut(&mut [f32]) {
    let mut clock = 0f32;
    let loaded_channels = channels.iter().map(|c| c.load()).collect::<Vec<_>>();
    move |data| {
        for samples in data.chunks_mut(2) {
            clock = (clock + 1.0) % sample_rate as f32;
            let valueL = loaded_channels
                .iter()
                .map(|c| c.play(clock, sample_rate) * c.pan[0])
                .sum::<f32>();
            let valueR = loaded_channels
                .iter()
                .map(|c| c.play(clock, sample_rate) * c.pan[1])
                .sum::<f32>();
            for (i, sample) in samples.iter_mut().enumerate() {
                *sample = [valueL, valueR][i];
            }
        }
    }
}
fn gen_sine_wave(freq: f32, clock: f32, volume: f32) -> f32 {
    (clock * freq).sin() * volume
}
fn gen_square_wave(freq: f32, clock: f32, volume: f32) -> f32 {
    (if (clock * freq).sin() > 0.0 {
        1.0
    } else {
        -1.0
    } * volume)
}
fn gen_triangle_wave(freq: f32, clock: f32, volume: f32) -> f32 {
    (if (clock * freq).sin() > 0.0 {
        1.0 - 2.0 * (clock * freq).sin()
    } else {
        -1.0 + 2.0 * (clock * freq).sin()
    } * volume)
}
fn gen_sawtooth_wave(freq: f32, clock: f32, volume: f32) -> f32 {
    (((freq * clock).tan().recip().atan()) / (std::f32::consts::PI / 2.0)) * volume
}

#[derive(Debug, Clone)]
struct Channel {
    freq: Option<f32>,
    volume: f32,
    wave: Option<WaveGenerator>,
    pan: [f32; 2],
    wave_sample: Option<Vec<f32>>,
}

impl Channel {
    fn new(freq: f32, volume: f32, pan: [f32; 2], wave: WaveGenerator) -> Self {
        Channel {
            freq: Some(freq),
            volume,
            wave: Some(wave),
            pan,
            wave_sample: None,
        }
    }
    fn new_with_sample(sample: Vec<f32>, volume: f32, pan: [f32; 2]) -> Self {
        Channel {
            freq: None,
            volume,
            wave: None,
            pan,
            wave_sample: Some(sample),
        }
    }
    fn play(&self, clock: f32, sample_rate: u32) -> f32 {
        if self.wave_sample == None {
            (self.wave.expect("Couldn't get channel wave"))(
                self.freq.expect("Couldn't get channel frequency") * std::f32::consts::PI
                    / sample_rate as f32,
                clock,
                self.volume,
            )
        } else {
            self.wave_sample
                .as_ref()
                .expect("Couldn't get channel data")[clock as usize
                % self
                    .wave_sample
                    .as_ref()
                    .expect("Couldn't get channel data")
                    .len()]
                * self.volume
        }
    }
}
