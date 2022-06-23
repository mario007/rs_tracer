use crate::vec::f32x3;
use crate::pixel_buffer::Color;
use crate::scene::{LightInterface, LightSample};


pub struct PointLight {
    intensity: Color,
    position: f32x3
}

impl PointLight {
    pub fn new(intensity: Color, position: f32x3) -> PointLight {
        PointLight { intensity, position }
    }
}

impl LightInterface for PointLight {
    fn illuminate(&self, hit: f32x3) -> LightSample {
        let direction_to_light = self.position - hit;
        let wi = direction_to_light.normalize();
        let intensity = self.intensity * direction_to_light.length_sqr().recip();
        let position = self.position;
        let pdfa = 1.0;
        let cos_theta = 1.0;
        LightSample { intensity, position, wi, pdfa, cos_theta}
    }
}
