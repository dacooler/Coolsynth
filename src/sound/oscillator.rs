use std::f64::consts::TAU;

pub trait Oscillator: Send{
    fn get_wave(&self, time: f64) -> f64;
}

impl Oscillator for SineOscillator{
    fn get_wave(&self, time: f64) -> f64{
        return (TAU*time*self.freq).sin();
    }
}
impl SineOscillator{
    pub fn new(freq: f64) -> Self{
        Self{freq}
    }
}
pub struct SineOscillator{
    freq: f64,
}

impl Oscillator for SquareOscillator{
    fn get_wave(&self, time: f64) -> f64{
        if ((time*self.freq) % 1.) > 0.5 {
            return -0.1;
        }
        else{
            return 0.1;
        }
    }
}
impl SquareOscillator{
    pub fn new(freq: f64) -> Self{
        Self{freq}
    }
}
pub struct SquareOscillator{
    freq: f64,
}


impl Oscillator for SawOscillator{
    fn get_wave(&self, time: f64) -> f64{
        return ((time*self.freq % 2.) -1.)/10.
    }
}
impl SawOscillator{
    pub fn new(freq: f64) -> Self{
        Self{freq}
    }
}
pub struct SawOscillator{
    freq: f64,
}