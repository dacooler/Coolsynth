use std::collections::LinkedList;
use std::f64::consts::TAU;
use std::sync::{Arc, Mutex};

use rodio::{ChannelCount, Float, SampleRate};
use rodio::{MixerDeviceSink, Player};
mod effector;
mod oscillator;

pub use crate::sound::effector::modulator::{Static, Envelope, Modulator, Attenuator};
use crate::sound::effector::{Effector, HpFilter, LpFilter, modulator::LFO, Delay};
use crate::sound::oscillator::{Oscillator, SawOscillator, SineOscillator, SquareOscillator};


impl rodio::source::Source for MasterAudio {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        ChannelCount::new(1).unwrap()
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        return SampleRate::new(44100).unwrap();
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}


impl Iterator for AudioGenerator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        self.time += 1.0 / 44100.0;
        let env = self.envelope.lock().unwrap().get_mod(self.time)?;
        let mut next = self.oscillator.get_wave(self.time);
        next *= env;
        next = self.effector.effect(next, self.time);
        return Some(next);

    }
}
pub struct AudioGenerator {
    time: f64,
    oscillator: Box<dyn Oscillator>,
    envelope: Arc<Mutex<Envelope>>,
    effector: Box<dyn Effector>

}

pub struct MasterAudio{
    pub sources: Arc<Mutex<Vec<AudioGenerator>>>,
    effector: Box<dyn Effector>,
    time: f64,
}

impl MasterAudio{
    pub fn new() -> Self{
        Self{ sources: Arc::new(Mutex::new(Vec::new())), effector: Box::new(Delay::new(Box::new(Static::new(12000.0)), 0.2, None)), time: 0.0 }
    }
}



impl Iterator for MasterAudio {
    type Item = Float;
    fn next(&mut self) -> Option<Self::Item> {
        let mut out = 0.0;
        self.time += 1./44100.0;
        self.sources.lock().unwrap().retain(|x| x.envelope.lock().unwrap().get_mod(x.time + 1. / 44100.0).is_some());
        for source in self.sources.lock().unwrap().iter_mut(){
            match source.next(){
                None => out += 0.0,
                Some(value) => out += value,
            }
        };
        return Some(self.effector.effect(out, self.time) as f32);
    }
}

pub fn play_note(mut sources: Arc<Mutex<Vec<AudioGenerator>>>, frequency: f64) -> Arc<Mutex<Envelope>> {
    // _stream must live as long as the sink
    let envelope = Envelope::new(
        0.01,
        0.2,
        0.50,
        0.2,
    );

    let envelope = Arc::new(Mutex::new(envelope));

    let source = AudioGenerator {
        time: 0.,
        oscillator: Box::new(SawOscillator::new(frequency)),
        effector:
            Box::new(LpFilter::new(
                Box::new(Attenuator::new(
                    Box::new(Envelope::new(
                        1.1, 0.2, 0.30, 0.2
                    )), 
                    3000., 0.0,
                )), 
                10.0, None,
            )),
        envelope: envelope.clone(),
    };
    
    sources.lock().unwrap().push(source);
    return envelope;
}
