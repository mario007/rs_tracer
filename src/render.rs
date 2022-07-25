use crate::ray::{Ray, offset_ray_origin};
use crate::scene::{SceneData, ShadingPoint};
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

// pub fn direct_lighting(ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {
//     let sp = match scene_data.intersect(ray, 1e30) {
//         Some(sp) => sp,
//         None => return Color::zero()
//     };

//     let mut acum_color = scene_data.get_emission(sp.shape_id);
//     for light in scene_data.lights.iter() {
//         let wo = -ray.direction;
//         let lgt_sample = match light.illuminate(sp.hitpoint, scene_data, rng) {
//             Some(lgt_sample) => lgt_sample,
//             None => continue
//         };
//         let wi = lgt_sample.wi;
//         if wi.dot(sp.normal) > 0.0 && wo.dot(sp.normal) > 0.0 {
//             let len_sqr = (sp.hitpoint - lgt_sample.position).length_sqr();
//             let bsdf_value = scene_data.eval_bsdf(&sp, wo, wi) * sp.normal.dot(wi);
//             let lgt_value = lgt_sample.intensity * lgt_sample.cos_theta;
//             let new_origin = offset_ray_origin(sp.hitpoint, sp.normal);
//             if scene_data.visible(new_origin, lgt_sample.position) {
//                 acum_color += lgt_value * bsdf_value * (len_sqr * lgt_sample.pdfa).recip();
//             }
//         }
//     }
//     acum_color
// }

// pub fn direct_lighting(ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {
//     let sp = match scene_data.intersect(ray, 1e30) {
//         Some(sp) => sp,
//         None => return Color::zero()
//     };

//     let mut acum_color = scene_data.get_emission(sp.shape_id);
//     let nlights = scene_data.lights.len();
//     let light_id = ((rng.rnd_f32() * nlights as f32) as usize).clamp(0, nlights - 1);
//     let light = &scene_data.lights[light_id];

//     let wo = -ray.direction;

//     let lgt_sample = match light.illuminate(sp.hitpoint, scene_data, rng) {
//         Some(lgt_sample) => lgt_sample,
//         None => return acum_color
//     };
//     let light_picking_pdf = 1.0 / nlights as f32;

//     let wi = lgt_sample.wi;
//     if wi.dot(sp.normal) > 0.0 && wo.dot(sp.normal) > 0.0 {
//         let len_sqr = (sp.hitpoint - lgt_sample.position).length_sqr();
//         let bsdf_value = scene_data.eval_bsdf(&sp, wo, wi) * sp.normal.dot(wi);
//         let lgt_value = lgt_sample.intensity * lgt_sample.cos_theta;
//         let new_origin = offset_ray_origin(sp.hitpoint, sp.normal);
//         if scene_data.visible(new_origin, lgt_sample.position) {
//             let light_pdf = light_picking_pdf * lgt_sample.pdfa;
//             acum_color += lgt_value * bsdf_value * (len_sqr * light_pdf).recip();
//         }
//     }
//     acum_color
// }

// pub fn direct_lighting(ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {

//     let sp = match scene_data.intersect(ray, 1e30) {
//         Some(sp) => sp,
//         None => return Color::zero()
//     };

//     let mut acum_color = scene_data.get_emission(sp.shape_id);
//     let wo = -ray.direction;

//     let bs = match scene_data.sample_bsdf(&sp, wo, rng) {
//         Some(bs) => bs,
//         None => return acum_color
//     };

//     let orgin = offset_ray_origin(sp.hitpoint, sp.normal);
//     let shadow_ray = Ray::new(orgin, bs.direction);

//     let lgt_sp= match scene_data.intersect(&shadow_ray, 1e30) {
//         Some(lgt_sp) => lgt_sp,
//         None => return acum_color
//     };

//     if sp.shape_id == lgt_sp.shape_id {
//         return acum_color
//     }

//     if !scene_data.is_emissive(lgt_sp.shape_id) {
//         return acum_color
//     }

//     let wi = bs.direction;
//     if wi.dot(sp.normal) > 0.0 && wo.dot(sp.normal) > 0.0 {
//         let bsdf_value = bs.color * sp.normal.dot(wi);
//         let emission = scene_data.get_emission(lgt_sp.shape_id);
//         acum_color += bsdf_value * emission * bs.pdfw.recip()
//     }
//     acum_color
// }

fn balance_heuristic(pdfa: f32, pdfb: f32) -> f32 {
    pdfa / (pdfa + pdfb)
}

pub fn direct_sample_light(sp: &ShadingPoint, ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {
    let wo = -ray.direction;
    let nlights = scene_data.lights.len();
    let light_id = ((rng.rnd_f32() * nlights as f32) as usize).clamp(0, nlights - 1);
    let light = &scene_data.lights[light_id];

    let lgt_sample = match light.illuminate(sp.hitpoint, scene_data, rng) {
        Some(lgt_sample) => lgt_sample,
        None => return Color::zero()
    };
    let light_picking_pdf = 1.0 / nlights as f32;

    let wi = lgt_sample.wi;
    if wi.dot(sp.normal) > 0.0 && wo.dot(sp.normal) > 0.0 {
        let len_sqr = (sp.hitpoint - lgt_sample.position).length_sqr();
        let bs = match scene_data.eval_bsdf(&sp, wo, wi) {
            Some(bs) => bs,
            None => return Color::zero()
        };
        let bsdf_value = bs.color * sp.normal.dot(wi);
        let lgt_value = lgt_sample.intensity * lgt_sample.cos_theta;
        let new_origin = offset_ray_origin(sp.hitpoint, sp.normal);
        if scene_data.visible(new_origin, lgt_sample.position) {
            let light_pdf = light_picking_pdf * lgt_sample.pdfa;
            let mut weight = 1.0;
            if !light.is_delta_light() {
                let bs_pdfa = bs.pdfw * lgt_sample.cos_theta * len_sqr.recip();
                weight = balance_heuristic(light_pdf, bs_pdfa);
            }
            return weight * lgt_value * bsdf_value * (len_sqr * light_pdf).recip();
        }
    }
    Color::zero()
}


pub fn direct_sample_bsdf(sp: &ShadingPoint, ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {
   
    let wo = -ray.direction;
    let origin = offset_ray_origin(sp.hitpoint, sp.normal);

    let bs = match scene_data.sample_bsdf(&sp, wo, rng) {
        Some(bs) => bs,
        None => return Color::zero()
    };

    let shadow_ray = Ray::new(origin, bs.direction);

    let lgt_sp= match scene_data.intersect(&shadow_ray, 1e30) {
        Some(lgt_sp) => lgt_sp,
        None => return Color::zero()
    };

    if sp.shape_id == lgt_sp.shape_id {
        return Color::zero()
    }

    if !scene_data.is_emissive(lgt_sp.shape_id) {
        return Color::zero()
    }

    let wi = bs.direction;
    if wi.dot(sp.normal) > 0.0 && wo.dot(sp.normal) > 0.0 {
        let bsdf_value = bs.color * sp.normal.dot(wi);
        let emission = scene_data.get_emission(lgt_sp.shape_id);
        if let Some(pdfa) = scene_data.geometry_pdfa(sp.hitpoint, &lgt_sp) {
            let cos_theta = lgt_sp.normal.dot(-wi).abs();
            let pdfw = pdfa * (sp.hitpoint - lgt_sp.hitpoint).length_sqr() * cos_theta.recip();
            let light_picking_pdf = 1.0 / scene_data.lights.len() as f32;
            let weight = balance_heuristic(bs.pdfw, pdfw * light_picking_pdf);
            return weight * (bsdf_value * emission) * bs.pdfw.recip();
        }
    }
    Color::zero()
}


pub fn direct_lighting(ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {

    let sp = match scene_data.intersect(ray, 1e30) {
        Some(sp) => sp,
        None => return Color::zero()
    };

    let mut acum_color = scene_data.get_emission(sp.shape_id);
    acum_color += direct_sample_light(&sp, ray, scene_data, rng);
    acum_color += direct_sample_bsdf(&sp, ray, scene_data, rng);

    acum_color
}


pub fn path_tracer(ray: &Ray, scene_data: &SceneData, rng: &mut PCGRng) -> Color {

    let mut sp = match scene_data.intersect(ray, 1e30) {
        Some(sp) => sp,
        None => return Color::zero()
    };

    let mut acum_color = scene_data.get_emission(sp.shape_id);

    let mut depth = 1;
    let max_depth = 10;
    let mut path = Color::one();
    let mut wo = -ray.direction;
    let threshold = 0.0001;

    loop {
        let bs = match scene_data.sample_bsdf(&sp, wo, rng) {
            Some(bs) => bs,
            None => break
        };

        let wi = bs.direction;
        let normal = sp.normal;

        let cos_theta = wi.dot(normal).abs();
        path = path * bs.color * (cos_theta / bs.pdfw);

        let origin = offset_ray_origin(sp.hitpoint, normal);
        let ray = Ray::new(origin, wi);

        sp = match scene_data.intersect(&ray, 1e30) {
            Some(sp) => sp,
            None => break
        };

        if scene_data.is_emissive(sp.shape_id) {
            if wi.dot(normal) > 0.0 && wo.dot(normal) > 0.0 {
                acum_color += path * scene_data.get_emission(sp.shape_id);
                break
            }
        }

        wo = -ray.direction;
        depth += 1;
        if depth == max_depth {
            break
        }

        if path.luminance() < threshold {
            break
        }

    }

    acum_color

}
