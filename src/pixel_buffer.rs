use crate::traits::{Zero, One};
use std::ops::{Add, AddAssign, Div, Mul};
use std::path::Path;
use std::error::Error;

extern crate image;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Color {
    fn to_rgb8(self) -> [u8; 3] {
        [
            (self.red * 256.0) as u8,
            (self.green * 256.0) as u8,
            (self.blue * 256.0) as u8,
        ]
    }

    pub fn luminance(self) -> f32 {
        self.red * 0.2126 + self.green * 0.7152 + self.blue * 0.0722
    }
}

impl Zero for Color {
    fn zero() -> Self {
        Self {red: 0.0, green: 0.0, blue: 0.0}
    }
    
}

impl One for Color {
    fn one() -> Self {
        Self {red: 1.0, green: 1.0, blue: 1.0}
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue
        }
    }
}

impl Mul for Color {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue
        }
    }
}

impl Div for Color {
    type Output = Color;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            red: self.red / rhs.red,
            green: self.green / rhs.green,
            blue: self.blue / rhs.blue
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Self::Output {
        Color {red: self.red * rhs, green: self.green * rhs, blue: self.blue * rhs}
    }
}

impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Self::Output {
        Color {red: self * rhs.red, green: self * rhs.green, blue: self * rhs.blue}
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PixelData {
    pub color: Color,
    pub weight: f32,
}

impl PixelData {
    fn get_color(&self) -> Color {
        if self.weight > 0.0 {
            return self.color * (1.0 / self.weight);
        }
        Color::zero()
    }
}

impl Zero for PixelData {
    fn zero() -> Self {
        PixelData { color: Color::zero(), weight: 0.0 }
    }
}

impl AddAssign for PixelData {
    
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            color: self.color + rhs.color,
            weight: self.weight + rhs.weight,
        }
    }
}

pub enum TMOType {
    Linear,
    Gamma,
    Reinhard,
}

fn tone_map(tmo_type: &TMOType, color: Color) -> Color {
    fn apply_gamma_correction(col: Color, gamma: f32) -> Color {
        Color { red: col.red.powf(gamma), green: col.green.powf(gamma), blue: col.blue.powf(gamma) }
    }

    let gamma: f32 = 1.0 / 2.2;

    match tmo_type {
        TMOType::Linear => color,
        TMOType::Gamma => apply_gamma_correction(color, gamma),
        TMOType::Reinhard => {
            let col = color / (Color::one() + color);
            apply_gamma_correction(col, gamma)
        }
    }
}

pub struct PixelBuffer {
    width: usize,
    height: usize,
    pixels: Vec<PixelData>
}

impl PixelBuffer {
    pub fn new(width: usize, height: usize) -> PixelBuffer {
        PixelBuffer { width, height, pixels: vec![PixelData::zero(); width * height] }
    }

    pub fn add_pixel(&mut self, x: usize, y: usize, pixel: &PixelData) {
        self.pixels[y * self.width + x] += *pixel;
    }

    fn save_as_rgb8<P: AsRef<Path>>(&self, path: P, tmo_type: &TMOType) -> Result<(), Box<dyn Error>> {

        let output: Vec<u8> = self.pixels.iter().flat_map(|pdata: &PixelData| {
            tone_map(tmo_type, pdata.get_color()).to_rgb8()
        }).collect();

        let result = image::save_buffer(path,
            &output[0..output.len()],
            self.width as u32,
            self.height as u32,
            image::ColorType::Rgb8);

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into())
        }
    }

    fn save_as_exr<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {

        let output: Vec<u8> = self.pixels.iter().flat_map(|pdata: &PixelData| {
            let c = pdata.get_color();
            [c.red.to_le_bytes(), c.green.to_le_bytes(), c.blue.to_le_bytes()].concat()
        }).collect();

        let result = image::save_buffer(path,
            &output[0..output.len()],
            self.width as u32,
            self.height as u32,
            image::ColorType::Rgb32F);

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(err.into())
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P, tmo_type: &TMOType) -> Result<(), Box<dyn Error>> {
        let ext = Path::new(path.as_ref()).extension();
        match ext {
            None => Err("There is no filename.".into()),
            Some(os_str) => match os_str.to_str() {
                Some("exr") => self.save_as_exr(path),
                _ => self.save_as_rgb8(path, tmo_type)
            }
        }
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use std::mem;

    fn fill_rect(buf: &mut PixelBuffer, pdata: &PixelData, x1: usize, x2: usize, y1: usize, y2: usize) {
        for j in y1..y2 {
            for i in x1..x2 {
                buf.add_pixel(i, j, pdata);
            }
        }
    }

    # [test]
    fn fill_image() {
        print!("Pixel data size {}\n", mem::size_of::<PixelData>());

        let red = PixelData{color: Color { red: 1.0, green: 0.0, blue: 0.0 }, weight: 1.0};
        let green = PixelData{color: Color { red: 0.0, green: 1.0, blue: 0.0 }, weight: 1.0};
        let blue = PixelData{color: Color { red: 0.0, green: 0.0, blue: 1.0 }, weight: 1.0};
        let mut buf = PixelBuffer::new(200, 300);
        fill_rect(&mut buf, &red, 0, 200, 0, 100);
        fill_rect(&mut buf, &green, 0, 200, 100, 200);
        fill_rect(&mut buf, &blue, 0, 200, 200, 300);
        buf.save("test.exr", &TMOType::Linear);

    }
}

