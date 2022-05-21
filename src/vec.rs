use std::ops::{Add, Sub, Mul, Neg};

// result = a * b - c * d
pub fn difference_of_products(a: f32, b: f32, c: f32, d: f32) -> f32 {
    let cd = c * d;
    let err = (-c).mul_add(d, cd);
    let dop = a.mul_add(b, -cd);
    dop + err
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct f32x3(pub f32, pub f32, pub f32);

impl f32x3 {
    pub fn dot(self, other: f32x3) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> f32x3 {
        let inv_len = 1.0 / self.length();
        f32x3(self.0 * inv_len, self.1 * inv_len, self.2 * inv_len)
    }

    pub fn cross(self, other: f32x3) -> f32x3 {
        f32x3(difference_of_products(self.1, other.2, self.2, other.1),
              difference_of_products(self.2, other.0, self.0, other.2),
              difference_of_products(self.0, other.1, self.1, other.0))
    }
}

impl Add for f32x3 {
    type Output = f32x3;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Sub for f32x3 {
    type Output = f32x3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl Mul<f32> for f32x3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Mul<f32x3> for f32 {
    type Output = f32x3;

    fn mul(self, rhs: f32x3) -> Self::Output {
        f32x3(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}

impl Neg for f32x3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2)
    }
}


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct f64x3(pub f64, pub f64, pub f64);

impl f64x3 {
    pub fn dot(self, other: f64x3) -> f64 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }

    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> f64x3 {
        let inv_len = 1.0 / self.length();
        f64x3(self.0 * inv_len, self.1 * inv_len, self.2 * inv_len)
    }

    pub fn cross(self, other: f64x3) -> f64x3 {
        f64x3(self.1 * other.2 - self.2 * other.1,
              self.2 * other.0 - self.0 * other.2,
              self.0 * other.1 - self.1 * other.0)
    }
}

impl Add for f64x3 {
    type Output = f64x3;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Sub for f64x3 {
    type Output = f64x3;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl Mul<f64> for f64x3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl Mul<f64x3> for f64 {
    type Output = f64x3;

    fn mul(self, rhs: f64x3) -> Self::Output {
        f64x3(self * rhs.0, self * rhs.1, self * rhs.2)
    }
}

impl Neg for f64x3 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0, -self.1, -self.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_of_products () {
        let a: f32 = 33962.035;
        let b: f32 = -30438.8;
        let c: f32 = 41563.4;
        let d: f32 = -24871.969;

        println!("Naive result {}", a * b - c * d);
        let diff = difference_of_products(a, b, c, d);
        println!("Kahan version {}", diff);
        println!("double version {}", a as f64 * b as f64 - c as f64 * d as f64);
    }

    #[test]
    fn f32_test () {
        let v = f32x3(1.0, 2.2, 0.5);
        let unit_v = v.normalize();
        assert_eq!(unit_v.length(), 1.0);

        let v2 = f32x3(0.5, 2.0, 0.8);
        let v3 = v2.cross(unit_v);
        assert_eq!(v3.dot(unit_v), 0.0);
        assert_eq!(v3.dot(v2), 0.0);

        let a = f32x3(1.0, 1.0, 1.0);
        let b = f32x3(1.0, 2.0, 3.0);
        let c = 2.0 * (-a) + b * 2.0;
        assert_eq!(c.0, 0.0);
        assert_eq!(c.1, 2.0);
        assert_eq!(c.2, 4.0);
    }

    #[test]
    fn f64_test () {
        let v = f64x3(1.0, 2.2, 0.5);
        let unit_v = v.normalize();
        assert_eq!(unit_v.length(), 1.0);

        let v2 = f64x3(0.5, 2.0, 0.8);
        let v3 = v2.cross(unit_v);
        assert_eq!(v3.dot(unit_v), 0.0);
        assert_eq!(v3.dot(v2), 0.0);

        let a = f64x3(1.0, 1.0, 1.0);
        let b = f64x3(1.0, 2.0, 3.0);
        let c = 2.0 * (-a) + b * 2.0;
        assert_eq!(c.0, 0.0);
        assert_eq!(c.1, 2.0);
        assert_eq!(c.2, 4.0);
    }
}
