
use crate::onb::ONB;
use crate::pixel_buffer::Color;
use crate::vec::f32x3;
use crate::scene::{BSDFInterface, BSDFEvalSample, BSDFSample};
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
    fn eval(&self, wo: f32x3, normal: f32x3, wi: f32x3) -> Option<BSDFEvalSample> {
        let color = self.reflectance * f32::consts::FRAC_1_PI;
        let pdfw = normal.dot(wi).abs() * f32::consts::FRAC_1_PI;
        Some(BSDFEvalSample{color, pdfw})
    }

    fn sample(&self, wo: f32x3, normal: f32x3, rng: &mut crate::pcg::PCGRng) -> Option<crate::scene::BSDFSample> {
        let u1 = rng.rnd_f32();
        let u2 = rng.rnd_f32();
        let term1 = 2.0 * f32::consts::PI * u1;
        let term2 = (1.0 - u2).sqrt();
        let x = term1.cos() * term2;
        let y = term1.sin() * term2;
        let z = u2.sqrt();

        let direction = ONB::from(normal).to_world(f32x3(x, y, z)).normalize();
        let color = self.reflectance * f32::consts::FRAC_1_PI;
        let pdfw = normal.dot(direction).abs() * f32::consts::FRAC_1_PI;
        if pdfw == 0.0 {
            return None
        }
        Some(BSDFSample{direction, color, pdfw})
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
    fn eval(&self, wo: f32x3, normal: f32x3, wi: f32x3) -> Option<BSDFEvalSample> {
        let color = self.reflectance * f32::consts::FRAC_1_PI;
        let pdfw = normal.dot(wi).abs() * f32::consts::FRAC_1_PI;
        Some(BSDFEvalSample{color, pdfw})
    }

    fn sample(&self, wo: f32x3, normal: f32x3, rng: &mut crate::pcg::PCGRng) -> Option<crate::scene::BSDFSample> {
        let u1 = rng.rnd_f32();
        let u2 = rng.rnd_f32();
        let term1 = 2.0 * f32::consts::PI * u1;
        let term2 = (1.0 - u2).sqrt();
        let x = term1.cos() * term2;
        let y = term1.sin() * term2;
        let z = u2.sqrt();

        let direction = ONB::from(normal).to_world(f32x3(x, y, z)).normalize();
        let color = self.reflectance * f32::consts::FRAC_1_PI;
        let pdfw = normal.dot(direction).abs() * f32::consts::FRAC_1_PI;
        if pdfw == 0.0 {
            return None
        }
        Some(BSDFSample{direction, color, pdfw})
    }

    fn is_emissive(&self) -> bool {
        true
    }

    fn emssion(&self) -> Color {
        self.emission
    }
}
