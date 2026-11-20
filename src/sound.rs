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

impl rodio::source::Source for MasterAudioIterator {
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
    pub sources: Arc<Mutex<Vec<AudioGenerator>>>,
    pub envelopes: Arc<Mutex<IndexMap<u8, Arc<Mutex<Envelope>>>>>,
    pub values: Arc<Mutex<Vec<f32>>>,
    effector: Vec<Box<dyn Effector>>,
    time: f64,
    cur_sample: Audio,
    done: bool,
}

impl MasterAudio {
    pub fn new(
        values: Arc<Mutex<Vec<f32>>>,
        toggles: Arc<Mutex<Vec<bool>>>,
        sources: Arc<Mutex<Vec<AudioGenerator>>>,
        envelopes: Arc<Mutex<IndexMap<u8, Arc<Mutex<Envelope>>>>>,
    ) -> Self {
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
            values: values.clone(),
        }
    }

    fn next(&mut self) -> Option<f32> {
        if self.done {
            self.cur_sample = Audio::new_m(0.0);
            self.time += 1. / 44100.0;
            self.sources.lock().unwrap().retain(|x| {
                x.envelope
                    .lock()
                    .unwrap()
                    .get_mod(x.time + 1. / 44100.0)
                    .is_some()
            });
            for source in self.sources.lock().unwrap().iter_mut() {
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
pub fn play_note(
    envelopes: Arc<Mutex<IndexMap<u8, Arc<Mutex<Envelope>>>>>,
    sources: Arc<Mutex<Vec<AudioGenerator>>>,
    values: Arc<Mutex<Vec<f32>>>,
    note: u8,
) {
    // _stream must live as long as the sink
    let envelope = Envelope::new_ub(
        Dynamic::new(values.clone(), 7),
        Dynamic::new(values.clone(), 8),
        Dynamic::new(values.clone(), 9),
        Dynamic::new(values.clone(), 10),
    );

    let envelope = Arc::new(Mutex::new(envelope));

    //let source = AudioGenerator { time: 0., oscillator: Box::new(SawOscillator::new(frequency)), envelope:envelope.clone(), effector: Box::new(HpFilter::new(Box::new(Static::new(0.)), None))};
    //
    let frequency = freq_from_note(note);

    let source = AudioGenerator {
        time: 0.,
        oscillator: Unison::new(vec![
            SawOscillator::new(frequency),
            SawOscillator::new(frequency + 10.),
        ]),
        effector: vec![Box::new(LpFilter::new(
            Attenuator::new(
                Envelope::new(
                    Dynamic::new(values.clone(), 3),
                    Dynamic::new(values.clone(), 4),
                    Dynamic::new(values.clone(), 5),
                    Dynamic::new(values.clone(), 6),
                ),
                Dynamic::new(values.clone(), 2),
                Dynamic::new(values.clone(), 0),
            ),
            Dynamic::new(values.clone(), 1),
            None,
        ))],
        envelope: envelope.clone(),
    };

    sources.lock().unwrap().push(source);
    envelopes.lock().unwrap().insert(note, envelope);
}
pub fn stop_playing(envelopes: Arc<Mutex<IndexMap<u8, Arc<Mutex<Envelope>>>>>, note: u8) {
    match envelopes.lock().unwrap().get(&note) {
        Some(envelope) => {
            envelope.lock().unwrap().sustained = false;
        }
        None => {}
    }
}

fn freq_from_note(note: u8) -> f64 {
    return 2. * (note - 12) as f64 / 12. * 440.;
}

pub struct MasterAudioIterator {
    pub master_audio: MasterAudio,
}

impl Iterator for MasterAudioIterator {
    type Item = Float;
    fn next(&mut self) -> Option<Self::Item> {
        self.master_audio.next()
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
