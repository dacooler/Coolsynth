use std::{collections::VecDeque, f64::consts::PI, sync::{Arc, Mutex}};

use eframe::egui::Key::L;

use crate::sound::effector::{audio::Audio, modulator::Modulator};

pub mod modulator;
pub mod audio;
pub trait Effector: Send{
    fn effect(&mut self, input: Audio, time: f64) -> Audio;
}

impl Effector for LpFilter {
    fn effect(&mut self, input: Audio, time: f64) -> Audio{
        let modulation = self.frequency.get_mod(time);
        let freq;
        let resonance = self.resonance.get_mod(time);
        match modulation{
            None => freq = 0.0,
            Some(frequency) => freq = frequency,
        }
        let res;
        match resonance{
            None => res = 0.0,
            Some(resonance) => res = resonance,
        }
        //let mut out = alpha * input + (1. - alpha) * self.state;
        let q = 1. / res;
        let w0: f64 = 2.*PI*(freq/44100.);
        let alpha = w0.sin() / 2. * q;
        let b0 = (1.0 - w0.cos())/2.;
        let b1 = 1.0 - w0.cos();
        let b2 = b0;
        let a0 = 1. + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1. - alpha;
        let out = (b0/a0)*input + (b1/a0)*self.state[0] + (b2/a0)*self.state[1] - (a1/a0)*self.state2[0] - (a2/a0)*self.state2[1];
        //let out = b0 * input + b1 * self.state[0] + b2 * self.state[1] - a1 * self.state2[0] - a2 * self.state2[1]; 
        self.state[2] = self.state[1];
        self.state[1] = self.state[0];
        self.state[0] = input;
        self.state2[2] = self.state2[1];
        self.state2[1] = self.state2[0];
        self.state2[0] = out;
        match &mut self.next{
            Some(effector) => return effector.effect(out, time),
            None => return out 
        }
    }
}
impl Effector for HpFilter {
    fn effect(&mut self, input: Audio, time: f64) -> Audio{
        let modulation = self.alpha.get_mod(time);
        match modulation {
            None => return Audio::new_m(0.),
            Some(alpha) => {
                self.state = alpha * input + (1. - alpha) * self.state;
                let out = input - self.state;
                return out 
            },
        }
    }
}
impl LpFilter {
    pub fn new<'a>(frequency: Box<dyn Modulator>, resonance: Box<dyn Modulator>, next: Option<Box<dyn Effector>>) -> Self{
        Self{ frequency, resonance, next, state: vec![Audio::new_m(0.0), Audio::new_m(0.0), Audio::new_m(0.0)], state2: vec![Audio::new_m(0.0), Audio::new_m(0.0), Audio::new_m(0.0)] }
    }
}
impl HpFilter {
    pub fn new(alpha: Box<dyn Modulator>, next: Option<Box<dyn Effector>>) -> Self{
        Self{ alpha, state: Audio::new_m(0.0) }
    }
}
pub struct HpFilter {
    alpha: Box<dyn Modulator>,
    state: Audio,
}
pub struct LpFilter {
    frequency: Box<dyn Modulator>,
    resonance: Box<dyn Modulator>,
    next: Option<Box<dyn Effector>>,
    state: Vec<Audio>,
    state2: Vec<Audio>,
}
pub struct Delay {
    time: Box<dyn Modulator>,
    feedback: f64,
    bbd: VecDeque<Audio>,
}
impl Delay {
    pub fn new(time: Box<dyn Modulator>, feedback: f64) -> Self{
        Self{ time, feedback, bbd: VecDeque::new() }
    }
}

impl Effector for Delay{
    fn effect(&mut self, input: Audio, time: f64) -> Audio{
       let delay_time = self.time.get_mod(time); 
       match delay_time{
        None => return Audio::new_m(0.),
        Some(delay) =>
        {
            let mut out;
            let steps = delay as usize;
            self.bbd.push_front(input);
            if steps > self.bbd.len(){
                out = Audio::new_m(0.0);
            }
            else if steps < self.bbd.len(){
                out = Option::expect(self.bbd.pop_back(), "Delay error");
                while steps < self.bbd.len(){
                    out = Option::expect(self.bbd.pop_back(), "Delay error");
                } 
            }
            else{
                out = Option::expect(self.bbd.pop_back(), "whoops");
                match self.bbd.front_mut(){
                    None => {},
                    Some(value) => {*value = *value + out * self.feedback},
                }
            }
            return out;
        },
       }
    }
}

pub struct Stereo{
    left: Vec<Box<dyn Effector>>,
    right: Vec<Box<dyn Effector>>,
}

impl Effector for Stereo{
    fn effect(&mut self, input: Audio, time: f64) -> Audio {
        let mut left = Audio::new_m(input.left);
        let mut right = Audio::new_m(input.right);
        for effect in &mut self.left{
            left = effect.effect(left, time);
        }
        for effect in &mut self.right{
            right = effect.effect(right, time);
        }
        return Audio::new(left.left, right.right); 
    }
}

impl Stereo{
    pub fn new(left: Vec<Box<dyn Effector>>, right: Vec<Box<dyn Effector>>) -> Box<Self>{
        Box::new(Self{ left: left, right: right})
    }
}

pub struct Distortion{
}

impl Effector for Distortion{
    fn effect(&mut self, input: Audio, time: f64) -> Audio {
        let outleft;
        let outright;
        if input.left > 0.0{
            outleft = input.left.powf(0.5);
        }
        else{
            let left = -input.left;
            outleft = -left.powf(0.5);
        }
        if input.right > 0.0{
            outright = input.right.powf(0.5);
        }
        else{
            let right = -input.right;
            outright = -right.powf(0.5);
        }

        let out = Audio::new(outleft, outright);
        return out 
    }
}

impl Distortion{
    pub fn new() -> Box<Self>{
        Box::new(Self{})
    }
}

impl Effector for Toggle{
    fn effect(&mut self, input: Audio, time: f64) -> Audio {
        if self.toggles.lock().unwrap()[self.index] {
            return self.effect.effect(input, time);
        }
        return input;
    }
}

pub struct Toggle{
    toggles: Arc<Mutex<Vec<bool>>>,
    index: usize,
    effect: Box<dyn Effector>,
}

impl Toggle{
    pub fn new(toggles: Arc<Mutex<Vec<bool>>>, index: usize, effect: Box<dyn Effector>) -> Box<Self>{
        Box::new(Self{ toggles, index, effect})
    }
}

pub struct Mixer{
    effector: Box<dyn Effector>,
    mix: Box<dyn Modulator>,
}

impl Mixer{
    pub fn new(effector: Box<dyn Effector>, mix: Box<dyn Modulator>,) -> Box<Self>{
        Box::new(Self{ effector, mix })
    }
}

impl Effector for Mixer{
    fn effect(&mut self, input: Audio, time: f64) -> Audio {
        let mix = self.mix.get_mod(time);
        match mix{
            Some(mix) => return input * (1. - mix) + self.effector.effect(input, time) * mix,
            None => return input,
        }

    }
}