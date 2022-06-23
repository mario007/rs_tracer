
use crate::vec::{f32x3, f64x3};

pub trait GeometryInterface {
    fn intersect(&self, origin: f64x3, direction: f64x3, tmax: f64) -> Option<f64>;
    fn normal(&self, hitpoint: f32x3) -> f32x3;
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

impl GeometryInterface for Sphere {
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

    fn normal(&self, hitpoint: f32x3) -> f32x3 {
        (hitpoint - self.position).normalize()
    }
}


pub fn ray_triangle(v0: f64x3, v1: f64x3, v2: f64x3,
                    origin: f64x3, direction: f64x3, tmax: f64) -> Option<f64> {
    
    let a = v0.0 - v1.0;
    let b = v0.0 - v2.0;
    let c = direction.0;
    let d = v0.0 - origin.0;
    let e = v0.1 - v1.1;
    let f = v0.1 - v2.1;
    let g = direction.1;
    let h = v0.1 - origin.1;
    let i = v0.2 - v1.2;
    let j = v0.2 - v2.2;
    let k = direction.2;
    let l = v0.2 - origin.2;
    
    let m = f * k - g * j;
    let n = h * k - g * l;
    let p = f * l - h * j;
    let q = g * i - e * k;
    let s = e * j - f * i;

    let temp3 = a * m + b * q +  c * s;

    if temp3 == 0.0 { return None }

    let inv_denom = 1.0 / temp3;
    let e1 = d * m - b * n - c * p;
    let beta = e1 * inv_denom;

    if beta < 0.0 { return None }

    let r = e * l - h * i;
    let e2 = a * n + d * q + c * r;
    let gamma = e2 * inv_denom;

    if gamma < 0.0 { return None }

    if beta + gamma > 1.0 { return None}

    let e3 = a * p - b * r + d * s;
    let t = e3 * inv_denom;

    if t < 0.0 || t > tmax {
        return None
    }
    Some(t)
}

pub struct Triangle {
    pub v0: f32x3,
    pub v1: f32x3,
    pub v2: f32x3
}

impl Triangle {
    pub fn new(v0: f32x3, v1: f32x3, v2: f32x3) -> Triangle {
        Triangle { v0, v1, v2 }
    }
}

impl GeometryInterface for Triangle {
    fn intersect(&self, origin: f64x3, direction: f64x3, tmax: f64) -> Option<f64> {
        ray_triangle(f64x3::from(self.v0), f64x3::from(self.v1), f64x3::from(self.v2), origin, direction, tmax)
    }

    fn normal(&self, _hitpoint: f32x3) -> f32x3 {
        (self.v1 - self.v0).cross(self.v2 - self.v0).normalize()
    }
}

pub struct Shape<T> {
    pub geometry: T,
    pub material_id: usize
}

impl<T> Shape<T> {
    pub fn new(geometry: T, material_id: usize) -> Self {
        Shape { geometry, material_id }
    }
}

impl<T: GeometryInterface + Sync + Send> GeometryInterface for Shape<T> {
    fn intersect(&self, origin: f64x3, direction: f64x3, tmax: f64) -> Option<f64> {
        self.geometry.intersect(origin, direction, tmax)
    }

    fn normal(&self, hitpoint: f32x3) -> f32x3 {
        self.geometry.normal(hitpoint)
    }
}
