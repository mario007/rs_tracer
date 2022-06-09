use crate::vec::f32x3;
use std::convert::From;


pub struct ONB {
    u: f32x3,
    v: f32x3,
    w: f32x3
}

impl ONB {
    pub fn to_world(&self, vec: f32x3) -> f32x3 {
        self.u * vec.0 + self.v * vec.1 + self.w * vec.2
    }
}

impl From<f32x3> for ONB {
    fn from(normal: f32x3) -> Self {
        if normal.2 < 0.0 {
            let a = 1.0 / (1.0 - normal.2);
            let b = normal.0 * normal.1 * a;
            let b1 = f32x3(1.0 - normal.0 * normal.0 * a, -b, normal.0);
            let b2 = f32x3(b, normal.1 * normal.1 * a - 1.0, -normal.1);
            Self {u: b1, v: b2, w: normal}
        }
        else {
            let a = 1.0 / (1.0 + normal.2);
            let b = -normal.0 * normal.1 * a;
            let b1 = f32x3(1.0 - normal.0 * normal.0 * a, b, -normal.0);
            let b2 = f32x3(b, 1.0 - normal.1 * normal.1 * a, -normal.1);
            Self {u: b1, v: b2, w: normal}
        }   
    }
}