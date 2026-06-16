use std::ops;

#[derive(Clone)]
pub struct Audio{
    pub left: f64,
    pub right: f64,
}

impl Copy for Audio { }

impl Audio{
    pub fn get_sum(self) -> f64{
        (self.left + self.right)/2.
    }
    
    pub fn new(left: f64, right: f64) -> Self{
        Self{ left, right }
    }
    pub fn new_m(mono: f64) -> Self{
        Self{ left: mono,  right: mono}
    }

}

impl ops::Mul<f64> for Audio {
    type Output = Audio;

    fn mul(self, mult: f64) -> Audio{
        Audio{left: self.left * mult, right: self.right * mult}
    }
}

impl ops::Mul<Audio> for f64{
    type Output = Audio;

    fn mul(self, audio: Audio) -> Audio{
        Audio{left: audio.left * self, right: audio.right * self}
    }
}
impl ops::Add<Audio> for Audio{
    type Output = Audio;

    fn add(self, audio: Audio) -> Audio{
        Audio{left: audio.left + self.left, right: audio.right + self.right}
    }
}
impl ops::Sub<Audio> for Audio{
    type Output = Audio;

    fn sub(self, audio: Audio) -> Audio{
        Audio{left: self.left - audio.left, right: self.right - audio.right}
    }
}