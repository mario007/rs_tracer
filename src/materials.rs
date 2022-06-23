
use crate::pixel_buffer::Color;
use crate::vec::f32x3;
use crate::scene::BSDFInterface;
use std::f32;

pub struct MatteMaterial {
    reflctance: Color
}

impl MatteMaterial {
    pub fn new(reflctance: Color) -> MatteMaterial {
        MatteMaterial {reflctance}
    }
}

impl BSDFInterface for MatteMaterial {
    fn eval(&self, wo: f32x3, normal: f32x3, wi: f32x3) -> Color {
        self.reflctance * f32::consts::FRAC_1_PI
    }
}
