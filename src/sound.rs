use std::f64::consts::TAU;
use std::sync::{Arc, Mutex};

use rodio::{ChannelCount, Float, SampleRate};
use rodio::{MixerDeviceSink, Player};
mod effector;
mod oscillator;

pub use crate::sound::effector::modulator::{Envelope, Modulator};
use crate::sound::effector::{Effector, HpFilter, LpFilter, modulator::LFO};
use crate::sound::oscillator::{Oscillator, SawOscillator, SineOscillator, SquareOscillator};


impl rodio::source::Source for AudioGenerator {
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
    type Item = Float;

    fn next(&mut self) -> Option<Self::Item> {
        self.time += 1.0 / 44100.0;
        let env = self.envelope.lock().unwrap().get_mod(self.time)?;
        let mut next = self.oscillator.get_wave(self.time);

        next *= env;
        next = self.effector.effect(next, self.time);
        return Some(next as f32);
    }
}
struct AudioGenerator {
    time: f64,
    oscillator: Box<dyn Oscillator>,
    envelope: Arc<Mutex<Envelope>>,
    effector: Box<dyn Effector>

}
pub fn play_note(handle: &mut MixerDeviceSink, frequency: f64) -> Arc<Mutex<Envelope>> {
    // _stream must live as long as the sink
    let envelope = Envelope::new(
        0.01,
        1.0,
        0.5,
        1.2,
    );

    let envelope = Arc::new(Mutex::new(envelope));

    let source = AudioGenerator {
        time: 0.,
        oscillator: Box::new(SawOscillator::new(frequency)),
        effector: Box::new(LpFilter::new(Box::new(Envelope::new(0.5, 1., 0.3, 1.)), None)),
        envelope: envelope.clone(),
    };
    let source2 = AudioGenerator {
        time: 0.,
        oscillator: Box::new(SawOscillator::new(frequency + 2.)),
        effector: Box::new(LpFilter::new(Box::new(Envelope::new(0.5, 1., 0.3, 1.)), None)),
        envelope: envelope.clone(),
    };
    handle.mixer().add(source);
    handle.mixer().add(source2);
    return envelope;
}
