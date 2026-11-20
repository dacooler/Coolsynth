use indexmap::IndexMap;
use rodio::{ChannelCount, Float, SampleRate};
use std::sync::{Arc, Mutex};
mod effector;
mod oscillator;

use crate::sound::effector::audio::Audio;
pub use crate::sound::effector::modulator::{Attenuator, Envelope, Modulator};
use crate::sound::effector::modulator::{Dynamic, Slew};
use crate::sound::effector::{Delay, Effector, LpFilter, modulator::LFO};
use crate::sound::effector::{Distortion, Feedback, Mixer, Stereo, Toggle};
use crate::sound::oscillator::{Oscillator, SawOscillator, Unison};
use arc_swap::ArcSwap;
use std::sync::mpsc::Receiver;

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
        for effect in &mut self.effector {
            next = effect.effect(next, self.time);
        }
        return Some(next);
    }
}
pub struct AudioGenerator {
    time: f64,
    oscillator: Box<dyn Oscillator>,
    envelope: Arc<Mutex<Envelope>>,
    effector: Vec<Box<dyn Effector>>,
}

pub struct MasterAudio {
    sources: Vec<AudioGenerator>,
    values: Arc<ArcSwap<Vec<f32>>>,
    recv: Receiver<(u8, u8, u8)>,
    effector: Vec<Box<dyn Effector>>,
    envelopes: IndexMap<u8, Arc<Mutex<Envelope>>>,
    time: f64,
    cur_sample: Audio,
    done: bool,
}

impl MasterAudio {
    pub fn new(
        values: Arc<ArcSwap<Vec<f32>>>,
        toggles: Arc<ArcSwap<Vec<bool>>>,
        recv: Receiver<(u8, u8, u8)>,
    ) -> Self {
        let envelopes = IndexMap::new();
        let sources = Vec::new();
        let delay = Toggle::new(
            toggles.clone(),
            1,
            Mixer::new(
                Stereo::new(
                    vec![Feedback::new(
                        Delay::new(
                            Slew::new(
                                Attenuator::new_s(Dynamic::new(values.clone(), 14), 44100., 200.),
                                0.5,
                            ),
                            0.0,
                        ),
                        Dynamic::new(values.clone(), 15),
                    )],
                    vec![Feedback::new(
                        Delay::new(
                            Slew::new(
                                Attenuator::new_s(Dynamic::new(values.clone(), 14), 44100., 0.),
                                0.5,
                            ),
                            0.0,
                        ),
                        Dynamic::new(values.clone(), 15),
                    )],
                ),
                Dynamic::new(values.clone(), 13),
            ),
        );
        let chorus = Toggle::new(
            toggles.clone(),
            0,
            Mixer::new(
                Stereo::new(
                    vec![Delay::new(
                        Attenuator::new_s(LFO::new(3.2), 400., 620.),
                        0.1,
                    )],
                    vec![Delay::new(
                        Attenuator::new_s(LFO::new(4.2), 400., 620.),
                        0.1,
                    )],
                ),
                Attenuator::new_s(Dynamic::new(values.clone(), 12), 0.5, 0.0),
            ),
        );
        let chorus2 = Toggle::new(
            toggles.clone(),
            0,
            Mixer::new(
                Stereo::new(
                    vec![Delay::new(
                        Attenuator::new_s(LFO::new(4.0), 400., 620.),
                        0.1,
                    )],
                    vec![Delay::new(
                        Attenuator::new_s(LFO::new(3.0), 400., 620.),
                        0.1,
                    )],
                ),
                Dynamic::new(values.clone(), 12),
            ),
        );
        let distortion = Mixer::new(Distortion::new(), Dynamic::new(values.clone(), 11));
        Self {
            sources: sources,
            effector: vec![distortion, chorus, chorus2, delay],
            time: 0.0,
            cur_sample: Audio::new_m(0.0),
            done: true,
            envelopes: envelopes,
            values: values,
            recv: recv,
        }
    }

    fn set_values(&mut self) {
        match self.recv.try_recv() {
            Ok(event) => {
                if event.2 == 144 {
                    self.play_note(event.0);
                }
                if event.2 == 128 {
                    self.stop_playing(event.0);
                }
            }
            Err(_error) => {}
        }
    }

    pub fn play_note(&mut self, note: u8) {
        // _stream must live as long as the sink
        let envelope = Envelope::new_ub(
            Dynamic::new(self.values.clone(), 7),
            Dynamic::new(self.values.clone(), 8),
            Dynamic::new(self.values.clone(), 9),
            Dynamic::new(self.values.clone(), 10),
        );

        let envelope = Arc::new(Mutex::new(envelope));

        //let source = AudioGenerator { time: 0., oscillator: Box::new(SawOscillator::new(frequency)), envelope:envelope.clone(), effector: Box::new(HpFilter::new(Box::new(Static::new(0.)), None))};
        //
        let frequency = freq_from_note(note);

        let source = AudioGenerator {
            time: 0.,
            oscillator: Unison::new(vec![SawOscillator::new(frequency)]),
            effector: vec![Box::new(LpFilter::new(
                Attenuator::new(
                    Envelope::new(
                        Dynamic::new(self.values.clone(), 3),
                        Dynamic::new(self.values.clone(), 4),
                        Dynamic::new(self.values.clone(), 5),
                        Dynamic::new(self.values.clone(), 6),
                    ),
                    Dynamic::new(self.values.clone(), 2),
                    Dynamic::new(self.values.clone(), 0),
                ),
                Dynamic::new(self.values.clone(), 1),
                None,
            ))],
            envelope: envelope.clone(),
        };

        self.sources.push(source);
        self.envelopes.insert(note, envelope);
        return;
    }
    pub fn stop_playing(&mut self, note: u8) {
        match self.envelopes.get(&note) {
            Some(envelope) => {
                envelope.lock().unwrap().sustained = false;
            }
            None => {}
        }
    }
}
fn freq_from_note(note: u8) -> f64 {
    let base: f64 = 2.;
    return 440. * base.powf((note as f64 - 69.) / 12.);
}

impl Iterator for MasterAudio {
    type Item = Float;
    fn next(&mut self) -> Option<f32> {
        self.set_values();
        if self.done {
            self.cur_sample = Audio::new_m(0.0);
            self.time += 1. / 44100.0;
            self.sources.retain(|x| {
                x.envelope
                    .lock()
                    .unwrap()
                    .get_mod(x.time + 1. / 44100.0)
                    .is_some()
            });
            for source in self.sources.iter_mut() {
                match source.next() {
                    None => {}
                    Some(value) => self.cur_sample = self.cur_sample + value,
                }
            }
            for effect in &mut self.effector {
                self.cur_sample = effect.effect(self.cur_sample, self.time);
            }
            self.done = false;
            return Some(self.cur_sample.left as f32);
        }
        self.done = true;
        return Some(self.cur_sample.right as f32);
    }
}

pub struct SynthValues {
    pub cutoff: f32,
    pub resonance: f32,
}

impl SynthValues {
    pub fn new(cutoff: f32, resonance: f32) -> Self {
        Self { cutoff, resonance }
    }
}
