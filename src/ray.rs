
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


fn offset_ray_origin(hit: f32x3, normal: f32x3) -> f32x3 {

    const fn int_scale() -> f32 {256.0}
    fn origin() -> f32 { 1.0 / 32.0}
    fn float_scale() -> f32 {1.0 / 65536.0}

    fn float_as_int(n: f32) -> i32 { i32::from_le_bytes(n.to_le_bytes())}
    fn int_as_float(n: i32) -> f32 { f32::from_le_bytes(n.to_le_bytes())}

    let of_i_x = (int_scale() * normal.0) as i32;
    let of_i_y = (int_scale() * normal.1) as i32;
    let of_i_z = (int_scale() * normal.2) as i32;

    let p_i_x: f32;
    let p_i_y: f32;
    let p_i_z: f32;

    if hit.0 < 0.0 {
        p_i_x = int_as_float(float_as_int(hit.0) - of_i_x);
    } else {
        p_i_x = int_as_float(float_as_int(hit.0) + of_i_x);
    }

    if hit.1 < 0.0 {
        p_i_y = int_as_float(float_as_int(hit.1) - of_i_y);
    } else {
        p_i_y = int_as_float(float_as_int(hit.1) + of_i_y);
    }

    if hit.2 < 0.0 {
        p_i_z = int_as_float(float_as_int(hit.2) - of_i_z);
    } else {
        p_i_z = int_as_float(float_as_int(hit.2) + of_i_z);
    }

    let rx: f32;
    let ry: f32;
    let rz: f32;

    if hit.0.abs() < origin() {
        rx = hit.0 + float_scale() * normal.0;
    } else {
        rx = p_i_x;
    }

    if hit.1.abs() < origin() {
        ry = hit.1 + float_scale() * normal.1;
    } else {
        ry = p_i_y;
    }

    if hit.2.abs() < origin() {
        rz = hit.2 + float_scale() * normal.2;
    } else {
        rz = p_i_z;
    }

    f32x3(rx, ry, rz)

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_offset() {
        let hit = f32x3(0.2, 0.3, 1.5);
        let normal = f32x3(1.0, 1.0, 1.0).normalize();
        println!("Offset point {:?}", offset_ray_origin(hit, normal));

        let hit = f32x3(112.0, 366.0, 885.0);
        println!("Offset point {:?}", offset_ray_origin(hit, normal));
    }
}
