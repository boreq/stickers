use crate::errors::Result;
use anyhow::anyhow;

const Reference_X: f32 = 109.850;
const Reference_Y: f32 = 100.000;
const Reference_Z: f32 = 35.585;

pub struct Color {
    color: SomeColor,
}

impl Color {
    pub fn rgb(&self) -> RGB {
        match &self.color {
            SomeColor::RGB(rgb) => rgb.clone(),
            SomeColor::YUV(yuv) => yuv.into(),
            SomeColor::LAB(lab) => {
                let xyz: XYZ = lab.into();
                let rgb: RGB = (&xyz).into();
                rgb
            }
        }
    }

    pub fn yuv(&self) -> YUV {
        match &self.color {
            SomeColor::RGB(rgb) => rgb.into(),
            SomeColor::YUV(yuv) => yuv.clone(),
            SomeColor::LAB(lab) => {
                let xyz: XYZ = lab.into();
                let rgb: RGB = (&xyz).into();
                let yuv: YUV = (&rgb).into();
                yuv
            }
        }
    }

    pub fn lab(&self) -> LAB {
        match &self.color {
            SomeColor::RGB(rgb) => {
                let xyz: XYZ = rgb.into();
                let lab: LAB = (&xyz).into();
                lab
            }
            SomeColor::YUV(yuv) => {
                let rgb: RGB = yuv.into();
                let xyz: XYZ = (&rgb).into();
                let lab: LAB = (&xyz).into();
                lab
            }
            SomeColor::LAB(lab) => lab.clone(),
        }
    }
}

impl From<RGB> for Color {
    fn from(value: RGB) -> Self {
        Self {
            color: SomeColor::RGB(value),
        }
    }
}

impl From<YUV> for Color {
    fn from(value: YUV) -> Self {
        Self {
            color: SomeColor::YUV(value),
        }
    }
}

impl From<LAB> for Color {
    fn from(value: LAB) -> Self {
        Self {
            color: SomeColor::LAB(value),
        }
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
        //sR, sG and sB (standard RGB) output range = 0 ÷ 255

        let var_X = value.x / 100.0;
        let var_Y = value.y / 100.0;
        let var_Z = value.z / 100.0;

        let mut var_R = var_X * 3.2406 + var_Y * -1.5372 + var_Z * -0.4986;
        let mut var_G = var_X * -0.9689 + var_Y * 1.8758 + var_Z * 0.0415;
        let mut var_B = var_X * 0.0557 + var_Y * -0.2040 + var_Z * 1.0570;

        if var_R > 0.0031308 {
            var_R = 1.055 * (var_R.powf(1.0 / 2.4)) - 0.055
        } else {
            var_R *= 12.92
        }
        if var_G > 0.0031308 {
            var_G = 1.055 * (var_G.powf(1.0 / 2.4)) - 0.055
        } else {
            var_G *= 12.92
        }
        if var_B > 0.0031308 {
            var_B = 1.055 * (var_B.powf(1.0 / 2.4)) - 0.055
        } else {
            var_B *= 12.92
        }

        let sR = var_R * 255.0;
        let sG = var_G * 255.0;
        let sB = var_B * 255.0;

        Self {
            r: sR as u8,
            g: sG as u8,
            b: sB as u8,
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
        let mut var_X = value.x / Reference_X;
        let mut var_Y = value.y / Reference_Y;
        let mut var_Z = value.z / Reference_Z;

        if var_X > 0.008856 {
            var_X = var_X.powf(1.0 / 3.0);
        } else {
            var_X = (7.787 * var_X) + (16.0 / 116.0);
        }

        if var_Y > 0.008856 {
            var_Y = var_Y.powf(1.0 / 3.0);
        } else {
            var_Y = (7.787 * var_Y) + (16.0 / 116.0);
        }

        if var_Z > 0.008856 {
            var_Z = var_Z.powf(1.0 / 3.0);
        } else {
            var_Z = (7.787 * var_Z) + (16.0 / 116.0);
        }

        let l = (116.0 * var_Y) - 16.0;
        let a = 500.0 * (var_X - var_Y);
        let b = 200.0 * (var_Y - var_Z);

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
        //sR, sG and sB (Standard RGB) input range = 0 ÷ 255
        //X, Y and Z output refer to a D65/2° standard illuminant.

        let mut var_R = value.r as f32 / 255.0;
        let mut var_G = value.g as f32 / 255.0;
        let mut var_B = value.b as f32 / 255.0;

        if var_R > 0.04045 {
            var_R = ((var_R + 0.055) / 1.055).powf(2.4)
        } else {
            var_R /= 12.92;
        }

        if var_G > 0.04045 {
            var_G = ((var_G + 0.055) / 1.055).powf(2.4);
        } else {
            var_G /= 12.92;
        }

        if var_B > 0.04045 {
            var_B = ((var_B + 0.055) / 1.055).powf(2.4);
        } else {
            var_B /= 12.92
        }

        var_R *= 100.0;
        var_G *= 100.0;
        var_B *= 100.0;

        let x = var_R * 0.4124 + var_G * 0.3576 + var_B * 0.1805;
        let y = var_R * 0.2126 + var_G * 0.7152 + var_B * 0.0722;
        let z = var_R * 0.0193 + var_G * 0.1192 + var_B * 0.9505;

        Self { x, y, z }
    }
}

impl From<&LAB> for XYZ {
    fn from(value: &LAB) -> Self {
        //Reference-X, Y and Z refer to specific illuminants and observers.
        //Common reference values are available below in this same page.

        let mut var_Y = (value.l + 16.0) / 116.0;
        let mut var_X = value.a / 500.0 + var_Y;
        let mut var_Z = var_Y - value.b / 200.0;

        if var_Y.powi(3) > 0.008856 {
            var_Y = var_Y.powi(3);
        } else {
            var_Y = (var_Y - 16.0 / 116.0) / 7.787;
        }

        if var_X.powi(3) > 0.008856 {
            var_X = var_X.powi(3);
        } else {
            var_X = (var_X - 16.0 / 116.0) / 7.787;
        }

        if var_Z.powi(3) > 0.008856 {
            var_Z = var_Z.powi(3);
        } else {
            var_Z = (var_Z - 16.0 / 116.0) / 7.787;
        }

        let x = var_X * Reference_X;
        let y = var_Y * Reference_Y;
        let z = var_Z * Reference_Z;

        Self { x, y, z }
    }
}

enum SomeColor {
    RGB(RGB),
    YUV(YUV),
    LAB(LAB),
}
