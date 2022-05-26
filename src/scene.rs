use std::default::Default;

use crate::{camera::PinholeCamera, vec::f32x3, ray::Ray};

extern crate num_cpus;

pub struct SceneData {
    width: usize,
    height: usize,
    nthreads: usize,
    camera: PinholeCamera
}

impl SceneData {
    pub fn image_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn get_nthreads(&self) -> usize {
        self.nthreads
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

}

impl Default for SceneData {
    fn default() -> Self {
        //Self { width: 200, height: 200, ntheads: num_cpus::get() }
        Self { width: 1024, height: 768, nthreads: 1, camera: PinholeCamera::default() }
    }
}
