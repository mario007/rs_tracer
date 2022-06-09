use std::default::Default;

use crate::camera::PinholeCamera;
use crate::vec::{f32x3, f64x3};
use crate::ray::Ray;
use crate::shapes::GeometryInterface;

extern crate num_cpus;

pub struct SceneData {
    width: usize,
    height: usize,
    nthreads: usize,
    samples_per_pixel: usize,
    camera: PinholeCamera,
    shapes: Vec<Box<dyn GeometryInterface + Send + Sync>>
}

pub struct ShadingPoint {
    pub t: f32,
    pub hitpoint: f32x3,
    pub normal: f32x3,
}

impl SceneData {
    pub fn image_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get_nthreads(&self) -> usize {
        self.nthreads
    }

    pub fn get_samples_per_pixel(&self) -> usize {
        return self.samples_per_pixel
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

    pub fn add_shape(&mut self, shape: Box<dyn GeometryInterface + Send + Sync>) {
        self.shapes.push(shape);
    }

    pub fn intersect(&self, ray: &Ray, tmax: f32) -> Option<ShadingPoint> {
        let origin = f64x3::from(ray.origin);
        let direction = f64x3::from(ray.direction);
        let mut cur_t = tmax as f64;
        let mut cur_shape_index = 0;
        for (index, shape) in self.shapes.iter().enumerate() {
            if let Some(t) = shape.intersect(origin, direction, cur_t) {
                if t < cur_t {
                    cur_t = t;
                    cur_shape_index = index;
                }
            }
        }
        if cur_t != tmax as f64 {
            let hitpoint = ray.origin + cur_t as f32 * ray.direction;
            let mut normal = self.shapes[cur_shape_index].normal(hitpoint);
            if normal.dot(-ray.direction) < 0.0 {
                normal = -normal;
            }
            return Some(ShadingPoint{t: cur_t as f32, hitpoint, normal});
        }
        None
    }

}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            width: 1024,
            height: 768,
            //ntheads: num_cpus::get()
            nthreads: 1,
            samples_per_pixel: 1,
            camera: PinholeCamera::default(),
            shapes: Vec::new()
        }
    }
}
