use std::collections::LinkedList;
use std::f64::consts::TAU;
use std::sync::{Arc, Mutex};

use rodio::{ChannelCount, Float, SampleRate};
use rodio::{MixerDeviceSink, Player};
mod effector;
mod oscillator;

use crate::sound::effector::modulator::Dynamic;
use crate::sound::effector::{Distortion, Mixer, Stereo, Toggle};
pub use crate::sound::effector::modulator::{Static, Envelope, Modulator, Attenuator};
use crate::sound::effector::{Effector, HpFilter, LpFilter, modulator::LFO, Delay};
use crate::sound::oscillator::{Oscillator, SawOscillator, SineOscillator, SquareOscillator, Unison};
use crate::sound::effector::audio::Audio;


impl rodio::source::Source for MasterAudio {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        ChannelCount::new(2).unwrap()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        return SampleRate::new(44100).unwrap();
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}


impl Iterator for AudioGenerator {
    type Item = Audio;

    fn next(&mut self) -> Option<Self::Item> {
        self.time += 1.0 / 44100.0;
        let env = self.envelope.lock().unwrap().get_mod(self.time)?;
        let mut next = self.oscillator.get_wave(self.time);
        next = next * env;
        for effect in &mut self.effector{
            next = effect.effect(next, self.time);
        }
        return Some(next);

    }
}
pub struct AudioGenerator {
    time: f64,
    oscillator: Box<dyn Oscillator>,
    envelope: Arc<Mutex<Envelope>>,
    effector: Vec<Box<dyn Effector>> 
}

pub struct MasterAudio{
    pub sources: Arc<Mutex<Vec<AudioGenerator>>>,
    effector: Vec<Box<dyn Effector>>,
    time: f64,
    cur_sample: Audio,
    done: bool,
}

impl MasterAudio{
    pub fn new(values: Arc<Mutex<Vec<f32>>>, toggles: Arc<Mutex<Vec<bool>>>) -> Self{
        let delay = Toggle::new(toggles.clone(), 1, Mixer::new(Stereo::new(
                vec![Box::new(Delay::new(Static::new(5500.0), 0.8))], 
                vec![Box::new(Delay::new(Static::new(5000.0), 0.8))],
            ), Dynamic::new(values.clone(), 13))); 
        let chorus = Toggle::new(toggles.clone(), 0, Mixer::new(
            Stereo::new(
                vec![Box::new(Delay::new(Attenuator::new_s(LFO::new(3.2), 400., 620.), 0.1))],
                vec![Box::new(Delay::new(Attenuator::new_s(LFO::new(4.2), 400., 620.), 0.1))],
            ), 
            Attenuator::new_s(Dynamic::new(values.clone(), 12), 0.5, 0.0))); 
        let chorus2 = Toggle::new(toggles.clone(), 0, 
            Mixer::new(Stereo::new(
                vec![Box::new(Delay::new(Attenuator::new_s(LFO::new(4.0), 400., 620.), 0.1))], 
                vec![Box::new(Delay::new(Attenuator::new_s(LFO::new(3.0), 400., 620.), 0.1))],
            ),
            Dynamic::new(values.clone(), 12))); 
        let distortion = 
            Mixer::new(Distortion::new(), Dynamic::new(values, 11));
        Self{ 
            sources: Arc::new(Mutex::new(Vec::new())),
            effector: vec![distortion, chorus, chorus2, delay],
            time: 0.0, 
            cur_sample: Audio::new_m(0.0), 
            done:true 
        }
    }
}



impl Iterator for MasterAudio {
    type Item = Float;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            self.cur_sample = Audio::new_m(0.0);
            self.time += 1./44100.0;
            self.sources.lock().unwrap().retain(|x| x.envelope.lock().unwrap().get_mod(x.time + 1. / 44100.0).is_some());
            for source in self.sources.lock().unwrap().iter_mut(){
                match source.next(){
                    None => {},
                    Some(value) => self.cur_sample = self.cur_sample + value,
                }
            };
            for effect in &mut self.effector{
                self.cur_sample = effect.effect(self.cur_sample, self.time);
            }
            self.done = false;
            return Some(self.cur_sample.left as f32);
        }
            self.done = true;
            return Some(self.cur_sample.right as f32);
    }
}

pub struct SynthValues{
    pub cutoff: f32,
    pub resonance: f32,
}

impl SynthValues{
    pub fn new(cutoff: f32, resonance: f32) -> Self{
        Self{cutoff, resonance}
    }
}

pub fn play_note(sources: Arc<Mutex<Vec<AudioGenerator>>>, frequency: f64, values: Arc<Mutex<Vec<f32>>>) -> Arc<Mutex<Envelope>> {
    // _stream must live as long as the sink
    let envelope = Envelope::new_ub(
        Dynamic::new(values.clone(), 7), 
        Dynamic::new(values.clone(), 8), 
        Dynamic::new(values.clone(), 9),
        Dynamic::new(values.clone(), 10) 
    );

    let envelope = Arc::new(Mutex::new(envelope));

    //let source = AudioGenerator { time: 0., oscillator: Box::new(SawOscillator::new(frequency)), envelope:envelope.clone(), effector: Box::new(HpFilter::new(Box::new(Static::new(0.)), None))};

    let source = AudioGenerator {
        time: 0.,
        oscillator: Unison::new(vec![SawOscillator::new(frequency), SawOscillator::new(frequency + 10.)]),
        effector:
            vec![
                Box::new(LpFilter::new(
                    Attenuator::new(
                        Envelope::new(
                            Dynamic::new(values.clone(), 3), 
                            Dynamic::new(values.clone(), 4), 
                            Dynamic::new(values.clone(), 5),
                            Dynamic::new(values.clone(), 6) 
                        ), 
                        Dynamic::new(values.clone(), 2), Dynamic::new(values.clone(), 0),
                    ), 
                    Dynamic::new(values.clone(), 1), None,
                )),
            ],
        envelope: envelope.clone(),
    };
    
    sources.lock().unwrap().push(source);
    return envelope;
}
