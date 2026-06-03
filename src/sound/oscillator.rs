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