use crate::ray::{Ray, offset_ray_origin};
use crate::scene::SceneData;
use crate::pcg::PCGRng;
use crate::pixel_buffer::Color;
use crate::traits::{Zero, One};
use crate::vec::f32x3;
use std::f32;
use crate::onb::ONB;


pub fn sample_hemisphere(normal: f32x3, u1: f32, u2: f32) -> (f32x3, f32) {
    let term2 = (1.0 - u1 * u1).sqrt();
    let x = (2.0 * f32::consts::PI * u2).cos() * term2;
    let y = (2.0 * f32::consts::PI * u2).sin() * term2;
    let z = u1;

    let direction = ONB::from(normal).to_world(f32x3(x, y, z)).normalize();
    let pdfw = 0.5 * f32::consts::FRAC_1_PI;
    (direction, pdfw)
}

pub fn ambient_occlusion(ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {

   if let Some(sp) = scene_data.intersect(ray, 1e30) {
        let (direction,pdfw) = sample_hemisphere(sp.normal, rng.rnd_f32(), rng.rnd_f32());
        let new_origin = offset_ray_origin(sp.hitpoint, sp.normal);
        let shadow_ray = Ray::new(new_origin, direction);
        let result = scene_data.intersect(&shadow_ray, 1e30);
        if result.is_none() {
            return Color::one() * f32::consts::FRAC_1_PI * sp.normal.dot(direction) * pdfw.recip();
        } else {
            return Color::zero();
        }
   } else {
       return Color::zero();
   }
}

pub fn direct_lighting(ray: &Ray, scene_data: &SceneData, _rng: &mut PCGRng) -> Color {
    let sp = match scene_data.intersect(ray, 1e30) {
        Some(sp) => sp,
        None => return Color::zero()
    };

    let mut acum_color = Color::zero();
    for light in scene_data.lights.iter() {
        let wo = -ray.direction;
        let lgt_sample = light.illuminate(sp.hitpoint);
        let wi = lgt_sample.wi;
        if wi.dot(sp.normal) > 0.0 && wo.dot(sp.normal) > 0.0 {
            let len_sqr = (sp.hitpoint - lgt_sample.position).length_sqr();
            let bsdf_value = scene_data.eval_bsdf(&sp, wo, wi) * sp.normal.dot(wi);
            let lgt_value = lgt_sample.intensity * lgt_sample.cos_theta;
            let new_origin = offset_ray_origin(sp.hitpoint, sp.normal);
            if scene_data.visible(new_origin, lgt_sample.position) {
                acum_color += lgt_value * bsdf_value * (len_sqr * lgt_sample.pdfa).recip();
            }
        }
    }

    acum_color
}
