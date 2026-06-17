use std::sync::{Arc, Mutex};


pub trait Modulator: Send{
    fn get_mod(&mut self, time: f64) -> Option<f64>;
}
impl Envelope {
    pub fn new(attack: Box<dyn Modulator>, decay: Box<dyn Modulator>, sustain: Box<dyn Modulator>, release: Box<dyn Modulator>) -> Box<Self>{
        Box::new(Self{ attack, decay, sustain, release, state: 0.0, sustained: true})
    }
    pub fn new_ub(attack: Box<dyn Modulator>, decay: Box<dyn Modulator>, sustain: Box<dyn Modulator>, release: Box<dyn Modulator>) -> Self{
        Self{ attack, decay, sustain, release, state: 0.0, sustained: true}
    }
}
impl Modulator for Envelope {
    fn get_mod(&mut self, time: f64) -> Option<f64> {
        let attack;
        let sustain;
        let decay;
        let release;
        match self.attack.get_mod(time){
            None => attack = 0.0,
            Some(x) => attack = x,
        }
        match self.decay.get_mod(time){
            None => decay = 0.0,
            Some(x) => decay = x,
        }
        match self.sustain.get_mod(time){
            None => sustain = 0.0,
            Some(x) => sustain = x,
        }
        match self.release.get_mod(time){
            None => release = 0.0,
            Some(x) => release = x,
        }
        if self.sustained {
            if time <= attack {
                self.state = time / attack;
                return Some(self.state);
            }
            else if time <= attack + decay {
                self.state = 1.0 - ((time - attack) / (decay + sustain));
                return Some(self.state);
            }
            return Some(sustain);
        }
        self.state -= 0.00001 / release;
        if self.state >= 0.0 {
            return Some(self.state);
        }
        else{
            return None;
        }
    }
}
pub struct Envelope {
    attack: Box<dyn Modulator>,
    decay: Box<dyn Modulator>,
    sustain: Box<dyn Modulator>,
    release: Box<dyn Modulator>,
    state: f64,
    pub sustained: bool,
}

impl Modulator for LFO{
    fn get_mod(&mut self, time: f64) -> Option<f64>{
        return Some(((time * self.freq).sin() + 1.)/2.);
    }
}
pub struct LFO{
    freq: f64
}

impl LFO {
    pub fn new(freq: f64) -> Box<Self>{
        Box::new(Self{ freq})
    }
}

impl Attenuator {
    pub fn new(modulator: Box<dyn Modulator>, strength: Box<dyn Modulator>, offset: Box<dyn Modulator>) -> Box<Self>{
        Box::new(Self{ modulator, strength, offset })
    }
    pub fn new_s(modulator: Box<dyn Modulator>, strength: f64, offset: f64) -> Box<Self>{
        Box::new(Self{ modulator, strength: Static::new(strength), offset: Static::new(offset) })
    }
}

pub struct Attenuator{
    modulator: Box<dyn Modulator>,
    strength: Box<dyn Modulator>,
    offset: Box<dyn Modulator>,
}

impl Modulator for Attenuator{
    fn get_mod(&mut self, time: f64) -> Option<f64> {
        let strength;
        let offset;
        match self.strength.get_mod(time){
            None => strength = 0.0,
            Some(str) => strength = str,
        }
        match self.offset.get_mod(time){
            None => offset = 0.0,
            Some(ofs) => offset = ofs,
        }
        return Some(self.modulator.get_mod(time)? * strength + offset);
    }
}

pub struct Static{
    value: f64,
}

impl Modulator for Static{
    fn get_mod(&mut self, time: f64) -> Option<f64> {
        return Some(self.value);
    }
}

impl Static{
    pub fn new(value: f64) -> Box<Self>{
        Box::new(Self{ value })
    }
}

impl Dynamic{
    pub fn new(map: Arc<Mutex<Vec<f32>>>, index: usize) -> Box<Self>{
        Box::new(Self{ map, index})
    }
}

pub struct Dynamic{
    map: Arc<Mutex<Vec<f32>>>,
    index: usize,
}

impl Modulator for Dynamic{
    fn get_mod(&mut self, time: f64) -> Option<f64> {
        return Some(self.map.lock().unwrap()[self.index] as f64);
    }
}