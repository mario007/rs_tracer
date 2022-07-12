
use crate::pixel_buffer::Color;
use crate::vec::f32x3;
use crate::scene::BSDFInterface;
use std::f32;

pub struct MatteMaterial {
    reflectance: Color
}

impl MatteMaterial {
    pub fn new(reflectance: Color) -> MatteMaterial {
        MatteMaterial {reflectance}
    }
}

impl BSDFInterface for MatteMaterial {
    fn eval(&self, wo: f32x3, normal: f32x3, wi: f32x3) -> Color {
        self.reflectance * f32::consts::FRAC_1_PI
    }
}

pub struct MatteEmissiveMaterial {
    reflectance: Color,
    emission: Color
}

impl MatteEmissiveMaterial {
    pub fn new(reflectance: Color, emission: Color) -> MatteEmissiveMaterial {
        MatteEmissiveMaterial { reflectance, emission }
    }
}

impl BSDFInterface for MatteEmissiveMaterial {
    fn eval(&self, wo: f32x3, normal: f32x3, wi: f32x3) -> Color {
        self.reflectance * f32::consts::FRAC_1_PI
    }

    fn is_emissive(&self) -> bool {
        true
    }

    fn emssion(&self) -> Color {
        self.emission
    }
}
