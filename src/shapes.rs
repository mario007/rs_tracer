use crate::vec::{f32x3, f64x3};

pub trait Intersect {
    fn intersect(&self, origin: f64x3, direction: f64x3, tmax: f64) -> Option<f64>;
}

pub struct Sphere {
    position: f32x3,
    radius: f32
}


impl Sphere {
    pub fn new(position: f32x3, radius: f32) -> Self {
        Self { position, radius }
    }
}

impl Intersect for Sphere {
    fn intersect(&self, origin: f64x3, direction: f64x3, tmax: f64) -> Option<f64> {
        let radius = self.radius as f64;
        let tmp = origin - f64x3::from(self.position);
        let a = direction.dot(direction);
        let b = 2.0 * tmp.dot(direction);
        let c = tmp.dot(tmp) - radius * radius;

        let disc = b * b - 4.0 * a * c;
        if disc < 0.0 {
            return None;
        } else {
            let e = disc.sqrt();
            let denom = 2.0 * a;
            let t = (-b - e) / denom;
            if t > 0.0 && t < tmax {
                return Some(t);
            }

            let t = (-b + e) / denom;
            if t > 0.0 && t < tmax {
                return Some(t);
            }
            None
        }
    }
}

