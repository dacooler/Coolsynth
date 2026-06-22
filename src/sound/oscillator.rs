use std::f64::consts::TAU;
use crate::sound::effector::audio::Audio;

pub trait Oscillator: Send{
    fn get_wave(&self, time: f64) -> Audio;
}

impl Oscillator for SineOscillator{
    fn get_wave(&self, time: f64) -> Audio{
        return Audio::new_m((TAU*time*self.freq).sin());
    }
}
impl SineOscillator{
    pub fn new(freq: f64) -> Box<Self>{
        Box::new(Self{freq})
    }
}
pub struct SineOscillator{
    freq: f64,
}

impl Oscillator for SquareOscillator{
    fn get_wave(&self, time: f64) -> Audio{
        if ((time*self.freq) % 1.) > 0.5 {
            return Audio::new_m(-0.1);
        }
        else{
            return Audio::new_m(0.1);
        }
    }
}
impl SquareOscillator{
    pub fn new(freq: f64) -> Box<Self>{
        Box::new(Self{freq})
    }
}
pub struct SquareOscillator{
    freq: f64,
}


impl Oscillator for SawOscillator{
    fn get_wave(&self, time: f64) -> Audio{
        return Audio::new_m(((time*self.freq % 2.) -1.)/10.)
    }
}
impl SawOscillator{
    pub fn new(freq: f64) -> Box<Self>{
        Box::new(Self{freq})
    }
}
pub struct SawOscillator{
    freq: f64,
}

pub struct Unison{
    oscillators: Vec<Box<dyn Oscillator>>,
}

impl Oscillator for Unison{
    fn get_wave(&self, time: f64) -> Audio {
        let mut out = Audio::new(0.0, 0.0);
        for oscillator in &self.oscillators{
            out = out + oscillator.get_wave(time); 
        }
        return out;
    }
}

impl Unison{
    pub fn new(oscillators: Vec<Box<dyn Oscillator>>) -> Box<Self>{
        Box::new(Self{ oscillators })
    }
}