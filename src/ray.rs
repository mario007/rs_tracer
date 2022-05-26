
use crate::vec::f32x3;

pub struct Ray {
    pub origin: f32x3,
    pub direction: f32x3
}

impl Ray {
    pub fn new(origin: f32x3, direction: f32x3) -> Ray {
        Ray { origin, direction }
    }
}
