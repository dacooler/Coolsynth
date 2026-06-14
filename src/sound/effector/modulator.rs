
pub trait Modulator: Send{
    fn get_mod(&mut self, time: f64) -> Option<f64>;
}
impl Envelope {
    pub fn new(attack: f64, decay: f64, sustain: f64, release: f64) -> Self{
        Self{ attack, decay, sustain, release, state: 0.0, sustained: true}
    }
}
impl Modulator for Envelope {
    fn get_mod(&mut self, time: f64) -> Option<f64> {
        if self.sustained {
            if time <= self.attack {
                self.state = time / self.attack;
                return Some(self.state);
            }
            else if time <= self.attack + self.decay {
                self.state = 1.0 - ((time - self.attack) / (self.decay + self.sustain));
                return Some(self.state);
            }
            return Some(self.sustain);
        }
        self.state -= 0.00001 / self.release;
        if self.state >= 0.0 {
            return Some(self.state);
        }
        else{
            return None;
        }
    }
}
pub struct Envelope {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
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
    pub fn new(freq: f64) -> Self{
        Self{ freq}
    }
}

impl Attenuator{
    pub fn new(modulator: Box<dyn Modulator>, strength: f64, offset: f64) -> Self{
        Self{ modulator, strength, offset }
    }
}

pub struct Attenuator{
    modulator: Box<dyn Modulator>,
    strength: f64,
    offset: f64,
}

impl Modulator for Attenuator{
    fn get_mod(&mut self, time: f64) -> Option<f64> {
        return Some(self.modulator.get_mod(time)? * self.strength + self.offset);
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
    pub fn new(value: f64) -> Self{
        Self{ value }
    }
}
