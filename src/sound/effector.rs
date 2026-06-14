use std::f64::consts::PI;

use crate::sound::effector::modulator::Modulator;

pub mod modulator;
pub trait Effector: Send{
    fn effect(&mut self, input: f64, time: f64) -> f64;
}

impl Effector for LpFilter {
    fn effect(&mut self, input: f64, time: f64) -> f64{
        let modulation = self.frequency.get_mod(time);
        match modulation {
            None => return 0.,
            Some(freq) => {
                //let mut out = alpha * input + (1. - alpha) * self.state;
                let resonance = 0.01;
                let w0: f64 = 2.*PI*(freq/44100.);
                let alpha = w0.sin() / 2. * resonance;
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

            },
        }
    }

}
impl Effector for HpFilter {
    fn effect(&mut self, input: f64, time: f64) -> f64{
        let modulation = self.alpha.get_mod(time);
        match modulation {
            None => return 0.,
            Some(alpha) => {
                self.state = alpha * input + (1. - alpha) * self.state;
                let out = input - self.state;
                match &mut self.next{
                    Some(effector) => return effector.effect(out, time),
                    None => return out 
                }
            },
        }
    }
}
impl LpFilter {
    pub fn new(frequency: Box<dyn Modulator>, next: Option<Box<dyn Effector>>) -> Self{
        Self{ frequency, next, state: vec![0.0, 0.0, 0.0], state2: vec![0.0, 0.0, 0.0] }
    }
}
impl HpFilter {
    pub fn new(alpha: Box<dyn Modulator>, next: Option<Box<dyn Effector>>) -> Self{
        Self{ alpha, next, state: 0.0 }
    }
}
pub struct HpFilter {
    alpha: Box<dyn Modulator>,
    next: Option<Box<dyn Effector>>,
    state: f64,
}
pub struct LpFilter {
    frequency: Box<dyn Modulator>,
    next: Option<Box<dyn Effector>>,
    state: Vec<f64>,
    state2: Vec<f64>,
}