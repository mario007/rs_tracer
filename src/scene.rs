use std::default::Default;

use crate::bvh::{BVHPrimitive, build_bottom_up_bvh, BVH};
use crate::camera::PinholeCamera;
use crate::lights::AreaLight;
use crate::pcg::PCGRng;
use crate::pixel_buffer::{Color, TMOType};
use crate::traits::Zero;
use crate::vec::{f32x3, f64x3};
use crate::ray::Ray;
use crate::shapes::{GeometryInterface, Shape};
use crate::bbox::AABB;

extern crate num_cpus;

pub struct BSDFEvalSample {
    pub color: Color,
    pub pdfw: f32
}

pub struct BSDFSample {
    pub direction: f32x3,
    pub color: Color,
    pub pdfw: f32
}

pub trait BSDFInterface {
    fn eval(&self, wo: f32x3, normal: f32x3, wi: f32x3) -> Option<BSDFEvalSample>;
    fn sample(&self, wo: f32x3, normal: f32x3, rng: &mut PCGRng) -> Option<BSDFSample>;
    fn is_emissive(&self) -> bool {
        false
    }
    fn emssion(&self) -> Color {
        Color::zero()
    }
}

pub struct LightSample {
    pub intensity: Color,
    pub position: f32x3,
    pub wi: f32x3,
    pub pdfa: f32,
    pub cos_theta: f32
}

pub struct ShapeSample {
    pub position: f32x3,
    pub pdfa: f32,
    pub normal: f32x3
}

pub trait LightInterface {
    fn illuminate(&self, hit: f32x3, scene_data: &SceneData, rng: &mut PCGRng) -> Option<LightSample>;
    fn is_delta_light(&self) -> bool;
    fn is_area_light(&self) -> bool {
        false
    }
}

pub enum RenderingAlgorithm {
    AmbientOcclusion,
    DirectLighting,
    PathTracer
}

pub struct SceneData {
    width: usize,
    height: usize,
    nthreads: usize,
    samples_per_pixel: usize,
    camera: PinholeCamera,
    shapes: Vec<Shape<Box<dyn GeometryInterface + Send + Sync>>>,
    materials: Vec<Box<dyn BSDFInterface + Send + Sync>>,
    pub lights: Vec<Box<dyn LightInterface + Send + Sync>>,
    pub rendering_algorithm: RenderingAlgorithm,
    output: String,
    tmo_type: TMOType,

    bbox_shapes: Vec<AABB>,
    bvh: Option<BVH>
}

pub struct ShadingPoint {
    pub t: f32,
    pub hitpoint: f32x3,
    pub normal: f32x3,
    material_id: usize,
    pub shape_id: usize
}

impl SceneData {
    pub fn image_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn set_image_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    pub fn get_nthreads(&self) -> usize {
        self.nthreads
    }

    pub fn set_nthreads(&mut self, nthreads: usize) {
        self.nthreads = nthreads
    }

    pub fn get_samples_per_pixel(&self) -> usize {
        return self.samples_per_pixel
    }

    pub fn set_samples_per_pixel(&mut self, samples_per_pixel: usize) {
        self.samples_per_pixel = samples_per_pixel;
    }

    pub fn set_rendering_algorithm(&mut self, rendering_algorithm: RenderingAlgorithm) {
        self.rendering_algorithm = rendering_algorithm
    }

    pub fn set_output_file(&mut self, file_path: String) {
        self.output = file_path
    }

    pub fn get_output_file(&self) -> String {
        self.output.clone()
    }

    pub fn set_tmo_type(&mut self, tmo_type: TMOType) {
        self.tmo_type = tmo_type
    }

    pub fn get_tmo_type(&self) -> &TMOType {
        &self.tmo_type
    }

    pub fn set_camera_pos(&mut self, position: f32x3) {
        self.camera.set_position(position);
    }

    pub fn set_camera_look_at(&mut self, look_at: f32x3) {
        self.camera.set_look_at(look_at);
    }

    pub fn set_camera_view_plane_distance(&mut self, view_plane_distance: f32) {
        self.camera.set_view_plane_distance(view_plane_distance);
    }

    pub fn set_camera_horizontal_fov(&mut self, fov: f32) {
        let half_width = self.width as f32 * 0.5;
        let view_plane_distance = half_width / (0.5 * fov).to_radians().tan();
        self.camera.set_view_plane_distance(view_plane_distance);
    }

    fn calculate_image_sample(&self, x: usize, y: usize, xp: f32, yp: f32) -> (f32, f32) {
        let img_x = x as f32 - self.width as f32 * 0.5 + xp;
        let img_y = y as f32 - self.height as f32 * 0.5 + yp;
        (img_x, img_y)
    }

    pub fn generate_ray(&self, x: usize, y: usize, xp: f32, yp: f32) -> Ray {
        let (img_x, img_y) = self.calculate_image_sample(x, y, xp, yp);
        self.camera.generate_ray(img_x, img_y)
    }

    pub fn add_shape(&mut self, shape: Shape<Box<dyn GeometryInterface + Send + Sync>>) {
        self.shapes.push(shape);
    }

    pub fn add_material(&mut self, material: Box<dyn BSDFInterface + Send + Sync>) -> usize {
        self.materials.push(material);
        self.materials.len() - 1
    }

    pub fn add_light(&mut self, light: Box<dyn LightInterface + Send + Sync>) {
        self.lights.push(light);
    }

    pub fn create_area_lights(&mut self) {
        self.lights.retain(|light| !light.is_area_light());
        for (shape_id, shape) in self.shapes.iter().enumerate() {
            if self.materials[shape.material_id].is_emissive() {
                self.lights.push(Box::new(AreaLight::new(shape_id)))
            }
        }
    }

    pub fn generate_shape_sample(&self, shape_id: usize, hit: f32x3, rng: &mut PCGRng) -> Option<ShapeSample> {
        self.shapes[shape_id].geometry.generate_sample(hit, rng)
    }

    pub fn get_emission(&self, shape_id: usize) -> Color {
        let material_id = self.shapes[shape_id].material_id;
        self.materials[material_id].emssion()
    }

    pub fn is_emissive(&self, shape_id: usize) -> bool {
        let material_id = self.shapes[shape_id].material_id;
        self.materials[material_id].is_emissive()
    }

    pub fn intersect_new(&self, ray: &Ray, tmax: f32) -> Option<ShadingPoint> {
        let isect = |prim: usize, origin: f64x3,
                                                 direction: f64x3, tmax: f64| -> Option<f64> {
            let shape = &self.shapes[prim];
            shape.geometry.intersect(origin, direction, tmax)
        };

        if let Some(bvh) = &self.bvh {
            if let Some(is) = bvh.intersection(&ray, tmax, &isect) {
                return Some(self.create_shading_point(&ray, is.t, is.primitive))
            }
        }
        None
    }

    fn create_shading_point(&self, ray: &Ray, t: f64, shape_id: usize) -> ShadingPoint {
        let shape = &self.shapes[shape_id];
        let hitpoint = ray.origin + t as f32 * ray.direction;
        let mut normal = shape.geometry.normal(hitpoint);
        
        if normal.dot(-ray.direction) < 0.0 {
            normal = -normal;
        }
        let material_id = shape.material_id;
        return ShadingPoint{t: t as f32, hitpoint, normal, material_id, shape_id};
    }

    pub fn intersect(&self, ray: &Ray, tmax: f32) -> Option<ShadingPoint> {
        let origin = f64x3::from(ray.origin);
        let direction = f64x3::from(ray.direction);
        let mut cur_t = tmax as f64;
        let mut cur_shape_index = 0;
        for index in 0..self.bbox_shapes.len() {
            let bbox = &self.bbox_shapes[index];
            if bbox.intersection(ray) {
                let shape = &self.shapes[index];
                if let Some(t) = shape.geometry.intersect(origin, direction, cur_t) {
                    if t < cur_t {
                        cur_t = t;
                        cur_shape_index = index;
                    }
                }
            }
        }
        // for (index, shape) in self.shapes.iter().enumerate() {
        //     if let Some(t) = shape.geometry.intersect(origin, direction, cur_t) {
        //         if t < cur_t {
        //             cur_t = t;
        //             cur_shape_index = index;
        //         }
        //     }
        // }
        if cur_t != tmax as f64 {
            let shape = &self.shapes[cur_shape_index];
            let hitpoint = ray.origin + cur_t as f32 * ray.direction;
            let mut normal = shape.geometry.normal(hitpoint);
            
            if normal.dot(-ray.direction) < 0.0 {
                normal = -normal;
            }
            let material_id = shape.material_id;
            return Some(ShadingPoint{t: cur_t as f32, hitpoint, normal, material_id, shape_id: cur_shape_index});
        }
        None
    }

    pub fn visible_new(&self, p0: f32x3, p1: f32x3) -> bool {
        let isect = |prim: usize, origin: f64x3,
                                                 direction: f64x3, tmax: f64| -> Option<f64> {
            let shape = &self.shapes[prim];
            shape.geometry.intersect(origin, direction, tmax)
        };

        if let Some(bvh) = &self.bvh {
            let direction = p1 - p0;
            let tmax = direction.length();
            let ray = Ray::new(p0, direction.normalize());
            return bvh.visible(&ray, tmax, &isect);
        }
        true
    }

    pub fn visible(&self, p0: f32x3, p1: f32x3) -> bool {
        let direction = p1 - p0;
        let tmax = direction.length();
        let ray = Ray::new(p0, direction.normalize());
        let origin = f64x3::from(ray.origin);
        let direction = f64x3::from(ray.direction);

        for index in 0..self.bbox_shapes.len() {
            let bbox = &self.bbox_shapes[index];
            if bbox.intersection(&ray) {
                let shape = &self.shapes[index];
                if let Some(_t) = shape.geometry.intersect(origin, direction, tmax as f64) {
                    return false
                }
            }
        }
        return true
    }

    pub fn eval_bsdf(&self, sp: &ShadingPoint, wo: f32x3, wi: f32x3) -> Option<BSDFEvalSample> {
        let material = &self.materials[sp.material_id];
        material.eval(wo, sp.normal, wi)
    }

    pub fn sample_bsdf(&self, sp: &ShadingPoint, wo: f32x3, rng: &mut PCGRng) -> Option<BSDFSample> {
        let material = &self.materials[sp.material_id];
        material.sample(wo, sp.normal, rng)
    }

    pub fn geometry_pdfa(&self, interaction_point: f32x3, sp: &ShadingPoint) -> Option<f32> {
        self.shapes[sp.shape_id].geometry.pdfa(interaction_point, sp.hitpoint)
    }

    pub fn prepare(&mut self) {
        self.bbox_shapes.clear();
        for shape in &self.shapes {
            self.bbox_shapes.push(shape.geometry.bbox());
        }

        // if !self.shapes.is_empty() {
        //     let mut prims = Vec::new();
        //     for (index, shape) in self.shapes.iter().enumerate() {
        //         prims.push(BVHPrimitive{bbox: shape.geometry.bbox(), primitive: index})
        //     }
        //     let bvh = build_bottom_up_bvh(&prims);
        //     self.bvh = Some(bvh);
        // }
    }


}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            nthreads: num_cpus::get(),
            samples_per_pixel: 1,
            camera: PinholeCamera::default(),
            shapes: Vec::new(),
            materials: Vec::new(),
            lights: Vec::new(),
            rendering_algorithm: RenderingAlgorithm::DirectLighting,
            output: "output.png".into(),
            tmo_type: TMOType::Gamma,
            bbox_shapes: Vec::new(),
            bvh: None
        }
    }
}
