use std::default::Default;
use crate::{vec::f32x3, ray::Ray};


pub struct PinholeCamera {
    eye: f32x3,
    look_at: f32x3,
    up: f32x3,
    view_plane_distance: f32,

    u: f32x3,
    v: f32x3,
    w: f32x3
}

impl PinholeCamera {
    fn new(eye: f32x3, look_at: f32x3, view_plane_distance: f32) -> PinholeCamera {
        let up = f32x3(0.0, 1.0, 0.0);

        let (u, v, w) = PinholeCamera::calculate_uvw(eye, look_at, up);
        PinholeCamera { eye, look_at, up, view_plane_distance, u, v, w }
    }

    fn calculate_uvw(eye: f32x3, look_at: f32x3, up: f32x3) -> (f32x3, f32x3, f32x3) {

        let u: f32x3;
        let v: f32x3;
        let w: f32x3;

        if eye.0 == look_at.0 && eye.2 == look_at.2 && eye.1 > look_at.1 {
            // camera looking vertically down
            u = f32x3(0.0, 0.0, 1.0);
            v = f32x3(1.0, 0.0, 0.0);
            w = f32x3(0.0, 1.0, 0.0);
        } else if eye.0 == look_at.0 && eye.2 == look_at.2 && eye.1 < look_at.1 {
            u = f32x3(1.0, 0.0, 0.0);
            v = f32x3(0.0, 0.0, 1.0);
            w = f32x3(0.0, -1.0, 0.0);
        } else {
            w = (eye - look_at).normalize();
            u =  up.cross(w).normalize();
            v = w.cross(u);
        }
        (u, v, w)
    }

    fn calculate_and_set_uvw(&mut self) {
        let (u, v, w) = PinholeCamera::calculate_uvw(self.eye, self.look_at, self.up);
        self.u = u;
        self.v = v;
        self.w = w;
    }

    pub fn set_position(&mut self, eye: f32x3) {
        self.eye = eye;
        self.calculate_and_set_uvw();
    }

    pub fn set_look_at(&mut self, look_at: f32x3) {
        self.look_at = look_at;
        self.calculate_and_set_uvw();
    }

    pub fn set_view_plane_distance(&mut self, view_plane_distance: f32) {
        self.view_plane_distance = view_plane_distance;
        self.calculate_and_set_uvw();
    }

    pub fn generate_ray(&self, x: f32, y: f32) -> Ray {
        let direction = (x * self.u + y * self.v - self.view_plane_distance * self.w).normalize();
        Ray::new(self.eye, direction)
    }
}

impl Default for PinholeCamera {
    fn default() -> Self {
        let eye = f32x3(0.0, 0.0, 0.0);
        let look_at = f32x3(0.0, 0.0, 5.0);
        Self::new(eye, look_at, 200.0)
    }
}

