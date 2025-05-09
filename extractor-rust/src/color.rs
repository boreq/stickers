use crate::errors::Result;
use anyhow::anyhow;
use image::Rgb;

const REFERENCE_X: f32 = 109.850;
const REFERENCE_Y: f32 = 100.000;
const REFERENCE_Z: f32 = 35.585;

pub struct Color {
    color: SomeColor,
}

impl Color {
    pub fn rgb(&self) -> RGB {
        match &self.color {
            SomeColor::Rgb(rgb) => rgb.clone(),
            SomeColor::Yuv(yuv) => yuv.into(),
            SomeColor::Lab(lab) => {
                let xyz: XYZ = lab.into();
                let rgb: RGB = (&xyz).into();
                rgb
            }
        }
    }

    pub fn yuv(&self) -> YUV {
        match &self.color {
            SomeColor::Rgb(rgb) => rgb.into(),
            SomeColor::Yuv(yuv) => yuv.clone(),
            SomeColor::Lab(lab) => {
                let xyz: XYZ = lab.into();
                let rgb: RGB = (&xyz).into();
                let yuv: YUV = (&rgb).into();
                yuv
            }
        }
    }

    pub fn lab(&self) -> LAB {
        match &self.color {
            SomeColor::Rgb(rgb) => {
                let xyz: XYZ = rgb.into();
                let lab: LAB = (&xyz).into();
                lab
            }
            SomeColor::Yuv(yuv) => {
                let rgb: RGB = yuv.into();
                let xyz: XYZ = (&rgb).into();
                let lab: LAB = (&xyz).into();
                lab
            }
            SomeColor::Lab(lab) => lab.clone(),
        }
    }
}

impl From<RGB> for Color {
    fn from(value: RGB) -> Self {
        Self {
            color: SomeColor::Rgb(value),
        }
    }
}

impl From<YUV> for Color {
    fn from(value: YUV) -> Self {
        Self {
            color: SomeColor::Yuv(value),
        }
    }
}

impl From<LAB> for Color {
    fn from(value: LAB) -> Self {
        Self {
            color: SomeColor::Lab(value),
        }
    }
}

impl From<Rgb<u8>> for Color {
    fn from(value: Rgb<u8>) -> Self {
        let [r, g, b] = value.0;
        RGB::new(r, g, b).into()
    }
}

#[derive(Debug, Clone)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn r(&self) -> u8 {
        self.r
    }

    pub fn g(&self) -> u8 {
        self.g
    }

    pub fn b(&self) -> u8 {
        self.b
    }
}

impl From<&YUV> for RGB {
    fn from(value: &YUV) -> Self {
        let r = value.y + 1.14 * value.v;
        let g = value.y - 0.395 * value.u * 0.581 * value.v;
        let b = value.y + 2.033 * value.u;
        RGB {
            r: (r * 255.0) as u8,
            g: (g * 255.0) as u8,
            b: (b * 255.0) as u8,
        }
    }
}

impl From<&XYZ> for RGB {
    fn from(value: &XYZ) -> Self {
        //X, Y and Z input refer to a D65/2° standard illuminant.
        //sr, sg and sb (standard RGB) output range = 0 ÷ 255

        let var_x = value.x / 100.0;
        let var_y = value.y / 100.0;
        let var_z = value.z / 100.0;

        let mut var_r = var_x * 3.2406 + var_y * -1.5372 + var_z * -0.4986;
        let mut var_g = var_x * -0.9689 + var_y * 1.8758 + var_z * 0.0415;
        let mut var_b = var_x * 0.0557 + var_y * -0.2040 + var_z * 1.0570;

        if var_r > 0.0031308 {
            var_r = 1.055 * (var_r.powf(1.0 / 2.4)) - 0.055
        } else {
            var_r *= 12.92
        }
        if var_g > 0.0031308 {
            var_g = 1.055 * (var_g.powf(1.0 / 2.4)) - 0.055
        } else {
            var_g *= 12.92
        }
        if var_b > 0.0031308 {
            var_b = 1.055 * (var_b.powf(1.0 / 2.4)) - 0.055
        } else {
            var_b *= 12.92
        }

        let sr = var_r * 255.0;
        let sg = var_g * 255.0;
        let sb = var_b * 255.0;

        Self {
            r: sr as u8,
            g: sg as u8,
            b: sb as u8,
        }
    }
}

const YUV_MAX_Y: f32 = 1.0;
const YUV_MAX_U: f32 = 0.436;
const YUV_MAX_V: f32 = 0.615;

#[derive(Debug, Clone)]
pub struct YUV {
    y: f32,
    u: f32,
    v: f32,
}

impl YUV {
    //fn from_rgb(pixel: &Rgb<u8>) -> YUV {
    //    let channels = pixel.channels();
    //}
    pub fn new(y: f32, u: f32, v: f32) -> Result<Self> {
        if y < 0.0 {
            return Err(anyhow!("y can't be negative"));
        }

        if y > YUV_MAX_Y {
            return Err(anyhow!("y can't be above {}", YUV_MAX_Y));
        }

        if u.abs() > YUV_MAX_U {
            return Err(anyhow!("u can't be above {}", YUV_MAX_U));
        }

        if v.abs() > YUV_MAX_V {
            return Err(anyhow!("v can't be above {}", YUV_MAX_V));
        }

        Ok(Self { y, u, v })
    }

    pub fn similar(&self, other: &Self, epsilon_y: f32, epsilon_uv: f32) -> bool {
        if (self.y - other.y).abs() > epsilon_y * YUV_MAX_Y {
            return false;
        }

        if (self.u - other.u).abs() > epsilon_uv * YUV_MAX_U {
            return false;
        }

        if (self.v - other.v).abs() > epsilon_uv * YUV_MAX_V {
            return false;
        }

        true
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn u(&self) -> f32 {
        self.u
    }

    pub fn v(&self) -> f32 {
        self.v
    }
}

impl From<&RGB> for YUV {
    fn from(value: &RGB) -> Self {
        let r = value.r as f32 / 255.0;
        let g = value.g as f32 / 255.0;
        let b = value.b as f32 / 255.0;
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        YUV {
            y,
            u: 0.492 * (b - y),
            v: 0.877 * (r - y),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LAB {
    l: f32,
    a: f32,
    b: f32,
}

impl LAB {
    pub fn new(l: f32, a: f32, b: f32) -> Result<Self> {
        Ok(Self { l, a, b })
    }

    pub fn distance(&self, other: &LAB) -> f32 {
        ((other.l - self.l).powi(2) + (other.a - self.a).powi(2) + (other.b - self.b).powi(2))
            .sqrt()
    }

    pub fn l(&self) -> f32 {
        self.l
    }

    pub fn a(&self) -> f32 {
        self.a
    }

    pub fn b(&self) -> f32 {
        self.b
    }
}

impl From<&XYZ> for LAB {
    fn from(value: &XYZ) -> Self {
        //Reference-X, Y and Z refer to specific illuminants and observers.
        //Common reference values are available below in this same page.
        let mut var_x = value.x / REFERENCE_X;
        let mut var_y = value.y / REFERENCE_Y;
        let mut var_z = value.z / REFERENCE_Z;

        if var_x > 0.008856 {
            var_x = var_x.powf(1.0 / 3.0);
        } else {
            var_x = (7.787 * var_x) + (16.0 / 116.0);
        }

        if var_y > 0.008856 {
            var_y = var_y.powf(1.0 / 3.0);
        } else {
            var_y = (7.787 * var_y) + (16.0 / 116.0);
        }

        if var_z > 0.008856 {
            var_z = var_z.powf(1.0 / 3.0);
        } else {
            var_z = (7.787 * var_z) + (16.0 / 116.0);
        }

        let l = (116.0 * var_y) - 16.0;
        let a = 500.0 * (var_x - var_y);
        let b = 200.0 * (var_y - var_z);

        Self { l, a, b }
    }
}

#[derive(Debug, Clone)]
pub struct XYZ {
    x: f32,
    y: f32,
    z: f32,
}

impl From<&RGB> for XYZ {
    fn from(value: &RGB) -> Self {
        //sr, sg and sb (Standard RGB) input range = 0 ÷ 255
        //X, Y and Z output refer to a D65/2° standard illuminant.

        let mut var_r = value.r as f32 / 255.0;
        let mut var_g = value.g as f32 / 255.0;
        let mut var_b = value.b as f32 / 255.0;

        if var_r > 0.04045 {
            var_r = ((var_r + 0.055) / 1.055).powf(2.4)
        } else {
            var_r /= 12.92;
        }

        if var_g > 0.04045 {
            var_g = ((var_g + 0.055) / 1.055).powf(2.4);
        } else {
            var_g /= 12.92;
        }

        if var_b > 0.04045 {
            var_b = ((var_b + 0.055) / 1.055).powf(2.4);
        } else {
            var_b /= 12.92
        }

        var_r *= 100.0;
        var_g *= 100.0;
        var_b *= 100.0;

        let x = var_r * 0.4124 + var_g * 0.3576 + var_b * 0.1805;
        let y = var_r * 0.2126 + var_g * 0.7152 + var_b * 0.0722;
        let z = var_r * 0.0193 + var_g * 0.1192 + var_b * 0.9505;

        Self { x, y, z }
    }
}

impl From<&LAB> for XYZ {
    fn from(value: &LAB) -> Self {
        //Reference-X, Y and Z refer to specific illuminants and observers.
        //Common reference values are available below in this same page.

        let mut var_y = (value.l + 16.0) / 116.0;
        let mut var_x = value.a / 500.0 + var_y;
        let mut var_z = var_y - value.b / 200.0;

        if var_y.powi(3) > 0.008856 {
            var_y = var_y.powi(3);
        } else {
            var_y = (var_y - 16.0 / 116.0) / 7.787;
        }

        if var_x.powi(3) > 0.008856 {
            var_x = var_x.powi(3);
        } else {
            var_x = (var_x - 16.0 / 116.0) / 7.787;
        }

        if var_z.powi(3) > 0.008856 {
            var_z = var_z.powi(3);
        } else {
            var_z = (var_z - 16.0 / 116.0) / 7.787;
        }

        let x = var_x * REFERENCE_X;
        let y = var_y * REFERENCE_Y;
        let z = var_z * REFERENCE_Z;

        Self { x, y, z }
    }
}

enum SomeColor {
    Rgb(RGB),
    Yuv(YUV),
    Lab(LAB),
}
