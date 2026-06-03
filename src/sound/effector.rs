use crate::sound::effector::modulator::Modulator;

pub mod modulator;
pub trait Effector: Send{
    fn effect(&mut self, input: f64, time: f64) -> f64;
}

impl Effector for LpFilter {
    fn effect(&mut self, input: f64, time: f64) -> f64{
        let modulation = self.alpha.get_mod(time);
        match modulation {
            None => return 0.,
            Some(alpha) => {
                let mut out = alpha * input + (1. - alpha) * self.state;
                self.state = out;
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
    pub fn new(alpha: Box<dyn Modulator>, next: Option<Box<dyn Effector>>) -> Self{
        Self{ alpha, next, state: 0.0 }
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
    alpha: Box<dyn Modulator>,
    next: Option<Box<dyn Effector>>,
    state: f64,
}