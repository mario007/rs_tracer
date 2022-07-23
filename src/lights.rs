use crate::pcg::PCGRng;
use crate::ray::offset_ray_origin;
use crate::vec::f32x3;
use crate::pixel_buffer::Color;
use crate::scene::{LightInterface, LightSample, SceneData};


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
    fn illuminate(&self, hit: f32x3, _scene_data: &SceneData, _rng: &mut PCGRng) -> Option<LightSample> {
        let direction_to_light = self.position - hit;
        let wi = direction_to_light.normalize();
        let intensity = self.intensity * direction_to_light.length_sqr().recip();
        let position = self.position;
        let pdfa = 1.0;
        let cos_theta = 1.0;
        Some(LightSample { intensity, position, wi, pdfa, cos_theta})
    }

    fn is_delta_light(&self) -> bool {
        true
    }
}

pub struct AreaLight {
    shape_id: usize
}

impl AreaLight {
    pub fn new(shape_id: usize) -> AreaLight {
        AreaLight { shape_id }
    }
}

impl LightInterface for AreaLight {
    fn is_area_light(&self) -> bool {
        true
    }

    fn illuminate(&self, hit: f32x3, scene_data: &SceneData, rng: &mut PCGRng) -> Option<LightSample> {
        let shp_sample = match scene_data.generate_shape_sample(self.shape_id, hit, rng) {
            Some(shp_sample) => shp_sample,
            None => return None
        };

        let position = offset_ray_origin(shp_sample.position, shp_sample.normal);
        let pdfa = shp_sample.pdfa;
        let normal = shp_sample.normal;

        let wi = (position - hit).normalize();
        let cos_theta = normal.dot(-wi);
        if cos_theta < 0.0 {
            return None
        }

        let intensity = scene_data.get_emission(self.shape_id);
        Some(LightSample{intensity, position, wi, pdfa, cos_theta})
    }

    fn is_delta_light(&self) -> bool {
        false
    }
}
