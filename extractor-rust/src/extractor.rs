use anyhow::anyhow;
use image::{GenericImageView, ImageReader, Pixel, Rgb, Rgba, RgbaImage};
use log::info;
use std::cmp;

use crate::errors::Result;

const MARKER_SCAN_STEP_IN_PERCENT: i32 = 1; // [%]
const MARKER_SCAN_STEPS: u32 = 30; // If this is 30 and step is 1 then 30% will be scanned.

pub fn extract(filepath: impl Into<String>) -> Result<()> {
    let filepath = filepath.into();

    info!("Opening image...");
    let img = ImageReader::open(filepath)?.decode()?;
    let mut img = img.to_rgba8();

    info!("Locating markers...");
    let markers = Markers::find(&img)?;

    info!("Coloring markers...");
    markers.top_left.color(&mut img);
    markers.top_right.color(&mut img);
    markers.bottom_left.color(&mut img);
    markers.bottom_right.color(&mut img);

    info!("Writing image...");
    img.save("empty.png")?;

    Ok(())
}

struct Markers {
    top_left: Area,
    top_right: Area,
    bottom_left: Area,
    bottom_right: Area,
}

impl Markers {
    fn find(img: &RgbaImage) -> Result<Markers> {
        let top_left = find_marker(img, &Corner::TopLeft)?;
        let top_right = find_marker(img, &Corner::TopRight)?;
        let bottom_left = find_marker(img, &Corner::BottomLeft)?;
        let bottom_right = find_marker(img, &Corner::BottomRight)?;

        if top_left.center().x > top_right.center().x {
            return Err(anyhow!("top left must be to the left of top right"));
        }

        if top_left.center().x > bottom_right.center().x {
            return Err(anyhow!("top left must be to the left of bottom right"));
        }

        if bottom_left.center().x > top_right.center().x {
            return Err(anyhow!("top left must be to the left of top right"));
        }

        if bottom_left.center().x > bottom_right.center().x {
            return Err(anyhow!("top left must be to the left of bottom right"));
        }

        if top_left.center().y > bottom_left.center().y {
            return Err(anyhow!("top left must be above bottom left"));
        }

        if top_left.center().y > bottom_right.center().y {
            return Err(anyhow!("top left must be above bottom right"));
        }

        if top_right.center().y > bottom_left.center().y {
            return Err(anyhow!("top right must be above bottom left"));
        }

        if top_right.center().y > bottom_right.center().y {
            return Err(anyhow!("top right must be above bottom right"));
        }

        Ok(Markers {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        })
    }
}

enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

fn find_marker(img: &RgbaImage, corner: &Corner) -> Result<Area> {
    let step_x: u32 = cmp::max(
        1,
        (MARKER_SCAN_STEP_IN_PERCENT as f32 / 100.0 * img.width() as f32) as u32,
    );
    let step_y: u32 = cmp::max(
        1,
        (MARKER_SCAN_STEP_IN_PERCENT as f32 / 100.0 * img.height() as f32) as u32,
    );

    for step_x_i in 0..MARKER_SCAN_STEPS {
        for step_y_i in 0..MARKER_SCAN_STEPS {
            let x = match corner {
                Corner::TopLeft => step_x_i * step_x,
                Corner::TopRight => img.width() - 1 - (step_x_i * step_x),
                Corner::BottomLeft => step_x_i * step_x,
                Corner::BottomRight => img.width() - 1 - (step_x_i * step_x),
            };
            let y = match corner {
                Corner::TopLeft => step_y_i * step_y,
                Corner::TopRight => step_y_i * step_y,
                Corner::BottomLeft => img.height() - 1 - (step_y_i * step_y),
                Corner::BottomRight => img.height() - 1 - (step_y_i * step_y),
            };

            if x >= img.width() || y >= img.height() {
                return Err(anyhow!("here is a nickel kid, get yourself a bigger image"));
            }

            if let Some(area) = flood_fill(img, x, y) {
                return Ok(area);
            }
        }
    }

    Err(anyhow!("not found"))
}

fn flood_fill(img: &RgbaImage, x: u32, y: u32) -> Option<Area> {
    let mut pixels = vec![];
    flood_fill_child(img, &XY { x, y }, &mut pixels);
    Area::from_pixels(pixels)
}

fn flood_fill_child(img: &RgbaImage, xy: &XY, pixels: &mut Vec<XY>) {
    if pixels.contains(xy) {
        return;
    }

    let pixel = img.get_pixel(xy.x, xy.y);
    let yuv = YUV::from_rgb(&pixel.to_rgb());

    if yuv.Y < 0.8 || yuv.U.abs() > 0.1 || yuv.V.abs() > 0.1 {
        return;
    }

    pixels.push(xy.clone());

    if xy.x > 0 {
        flood_fill_child(
            img,
            &XY {
                x: xy.x - 1,
                y: xy.y,
            },
            pixels,
        );
    }

    if xy.y > 0 {
        flood_fill_child(
            img,
            &XY {
                x: xy.x,
                y: xy.y - 1,
            },
            pixels,
        );
    }

    if xy.x < img.width() - 1 {
        flood_fill_child(
            img,
            &XY {
                x: xy.x + 1,
                y: xy.y,
            },
            pixels,
        );
    }

    if xy.y < img.height() - 1 {
        flood_fill_child(
            img,
            &XY {
                x: xy.x,
                y: xy.y + 1,
            },
            pixels,
        );
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
struct XY {
    x: u32,
    y: u32,
}

#[derive(Debug)]
struct YUV {
    Y: f32,
    U: f32,
    V: f32,
}

impl YUV {
    fn from_rgb(pixel: &Rgb<u8>) -> YUV {
        let channels = pixel.channels();
        let r = channels[0] as f32 / 256.0;
        let g = channels[1] as f32 / 256.0;
        let b = channels[2] as f32 / 256.0;
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        YUV {
            Y: y,
            U: 0.492 * (b - y),
            V: 0.877 * (r - y),
        }
    }
}

struct Area {
    top: u32,
    left: u32,
    width: u32,
    height: u32,
}

impl Area {
    fn from_pixels(pixels: Vec<XY>) -> Option<Area> {
        if pixels.is_empty() {
            return None;
        }

        let top = pixels.iter().map(|v| v.y).min().unwrap();
        let bottom = pixels.iter().map(|v| v.y).max().unwrap();
        let left = pixels.iter().map(|v| v.x).min().unwrap();
        let right = pixels.iter().map(|v| v.x).max().unwrap();

        Some(Area {
            top,
            left,
            width: right - left,
            height: bottom - top,
        })
    }

    fn right(&self) -> u32 {
        self.left + self.width
    }

    fn bottom(&self) -> u32 {
        self.top + self.height
    }

    fn center(&self) -> XY {
        XY {
            x: self.left + self.width / 2,
            y: self.top + self.height / 2,
        }
    }

    fn color(&self, img: &mut RgbaImage) {
        for x in self.left..self.right() {
            for y in self.top..self.bottom() {
                img.put_pixel(x, y, Rgb([255, 0, 0]).to_rgba());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() -> Result<()> {
        let rgb = Rgb([255, 255, 255]);
        println!("white {:?}", YUV::from_rgb(&rgb));

        let rgb = Rgb([0, 0, 0]);
        println!("black {:?}", YUV::from_rgb(&rgb));

        Ok(())
    }
}
