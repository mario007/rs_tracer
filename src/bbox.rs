use crate::vec::f32x3;
use crate::ray::Ray;
use std::f32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AABB {
    pub min: f32x3,
    pub max: f32x3
}

impl AABB {
    pub fn new(min: f32x3, max: f32x3) -> AABB {
        AABB { min, max }
    }

    // pub fn intersection(&self, ray: &Ray) -> bool {

    //     fn min(x: f32, y: f32) -> f32 {
    //         if x < y {x} else {y}
    //     }

    //     fn max(x: f32, y: f32) -> f32 {
    //         if x > y {x} else {y}
    //     }

    //     let mut tmin = 0.0;
    //     let mut tmax = f32::INFINITY;

    //     let t1 = (self.min.0 - ray.origin.0) * ray.inv_dir.0;
    //     let t2 = (self.max.0 - ray.origin.0) * ray.inv_dir.0;

    //     tmin = min(max(t1, tmin), max(t2, tmin));
    //     tmax = max(min(t1, tmax), min(t2, tmax));

    //     let t1 = (self.min.1 - ray.origin.1) * ray.inv_dir.1;
    //     let t2 = (self.max.1 - ray.origin.1) * ray.inv_dir.1;

    //     tmin = min(max(t1, tmin), max(t2, tmin));
    //     tmax = max(min(t1, tmax), min(t2, tmax));

    //     let t1 = (self.min.2 - ray.origin.2) * ray.inv_dir.2;
    //     let t2 = (self.max.2 - ray.origin.2) * ray.inv_dir.2;

    //     tmin = min(max(t1, tmin), max(t2, tmin));
    //     tmax = max(min(t1, tmax), min(t2, tmax));

    //     tmin <= tmax
    // }

    pub fn intersection(&self, ray: &Ray) -> bool {

        fn min(x: f32, y: f32) -> f32 {
            if x < y {x} else {y}
        }

        fn max(x: f32, y: f32) -> f32 {
            if x > y {x} else {y}
        }

        let mut tmin = 0.0;
        let mut tmax = f32::INFINITY;

        let d_min = self.min - ray.origin;
        let d_max = self.max - ray.origin;
        let inv_x = ray.inv_dir.0;
        let inv_y = ray.inv_dir.1;
        let inv_z = ray.inv_dir.2;

        let t1 = d_min.0 * inv_x;
        let t2 = d_max.0 * inv_x;

        tmin = min(max(t1, tmin), max(t2, tmin));
        tmax = max(min(t1, tmax), min(t2, tmax));

        let t1 = d_min.1 * inv_y;
        let t2 = d_max.1 * inv_y;

        tmin = min(max(t1, tmin), max(t2, tmin));
        tmax = max(min(t1, tmax), min(t2, tmax));

        let t1 = d_min.2 * inv_z;
        let t2 = d_max.2 * inv_z;

        tmin = min(max(t1, tmin), max(t2, tmin));
        tmax = max(min(t1, tmax), min(t2, tmax));

        tmin <= tmax
    }

    pub fn merge(&self, bbox: &AABB) -> AABB {
        AABB::new(self.min.min(bbox.min), self.max.max(bbox.max))
    }

    pub fn area(&self) -> f32 {
        let a1 = (self.max.0 - self.min.0) * (self.max.1 - self.min.1);
        let a2 = (self.max.1 - self.min.1) * (self.max.2 - self.min.2);
        let a3 = (self.max.2 - self.min.2) * (self.max.0 - self.min.0);
        (a1 + a2 + a3) * 2.0
    }
}
