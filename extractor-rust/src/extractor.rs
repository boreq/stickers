use crate::errors::Result;
use anyhow::anyhow;
use image::{Pixel, Rgb, Rgba, RgbaImage};
use std::{
    cmp,
    collections::{HashMap, HashSet},
};

const MARKER_SCAN_STEP_IN_PERCENT: i32 = 1; // [%]
const MARKER_SCAN_STEPS: u32 = 30; // If this is 30 and step is 1 then 30% will be scanned.
const BACKGROUND_ANALYSIS_STEPS: usize = 10;

// Marker must be at least 0.001% of the total image in pixel count.
const MARKER_THRESHOLD: f32 = 0.0001;

// Consider stickers to be in the same column if their
// centers are this far away.
const SNAP_STICKERS_THRESHOLD: f32 = 0.2;

pub const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

pub struct Markers {
    top_left: Area,
    top_right: Area,
    bottom_left: Area,
    bottom_right: Area,
}

impl Markers {
    pub fn find(img: &RgbaImage) -> Result<Markers> {
        let top_left = Markers::find_marker(img, &Corner::TopLeft)?;
        let top_right = Markers::find_marker(img, &Corner::TopRight)?;
        let bottom_left = Markers::find_marker(img, &Corner::BottomLeft)?;
        let bottom_right = Markers::find_marker(img, &Corner::BottomRight)?;

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

    fn find_marker(img: &RgbaImage, corner: &Corner) -> Result<Area> {
        let step_x: u32 = cmp::max(
            1,
            (MARKER_SCAN_STEP_IN_PERCENT as f32 / 100.0 * img.width() as f32) as u32,
        );
        let step_y: u32 = cmp::max(
            1,
            (MARKER_SCAN_STEP_IN_PERCENT as f32 / 100.0 * img.height() as f32) as u32,
        );

        let match_color =
            |_xy: &XY, yuv: &YUV| yuv.y > 0.7 && yuv.u.abs() < 0.15 && yuv.v.abs() < 0.15;

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

                let pixels = flood_fill(img, XY { x, y }, match_color);
                if !pixels.is_empty()
                    && is_at_least_this_much_of_image(pixels.len(), img, MARKER_THRESHOLD)
                {
                    let area = Area::from_pixels(pixels).unwrap();
                    return Ok(area);
                }
            }
        }

        Err(anyhow!("not found"))
    }

    pub fn middle_of_top_edge(&self) -> XY {
        let x = (self.top_left.center().x + self.top_right.center().x) / 2;
        let y = (self.top_left.center().y + self.top_right.center().y) / 2;
        XY { x, y }
    }

    pub fn markers(&self) -> Vec<&Area> {
        vec![
            &self.top_left,
            &self.top_right,
            &self.bottom_left,
            &self.bottom_right,
        ]
    }

    pub fn top_left(&self) -> &Area {
        &self.top_left
    }

    pub fn top_right(&self) -> &Area {
        &self.top_right
    }

    pub fn bottom_left(&self) -> &Area {
        &self.bottom_left
    }

    pub fn bottom_right(&self) -> &Area {
        &self.bottom_right
    }
}

pub struct Background {
    areas: HashMap<Area, YUV>,
}

impl Background {
    pub fn analyse(img: &RgbaImage, markers: &Markers) -> Result<Background> {
        let mut areas = HashMap::new();

        let marker_width = markers.top_left.width;
        let marker_height = markers.top_left.height;

        let iter_top = EdgeIterator::new(
            markers.top_left.center(),
            markers.top_right.center(),
            BACKGROUND_ANALYSIS_STEPS,
        )?;

        let iter_bottom = EdgeIterator::new(
            markers.bottom_left.center(),
            markers.bottom_right.center(),
            BACKGROUND_ANALYSIS_STEPS,
        )?;

        let iter_left = EdgeIterator::new(
            markers.top_left.center(),
            markers.bottom_left.center(),
            BACKGROUND_ANALYSIS_STEPS,
        )?;

        let iter_right = EdgeIterator::new(
            markers.top_right.center(),
            markers.bottom_right.center(),
            BACKGROUND_ANALYSIS_STEPS,
        )?;

        for (i, xy) in iter_top
            .chain(iter_bottom)
            .chain(iter_left)
            .chain(iter_right)
        {
            if i == 0 || i == BACKGROUND_ANALYSIS_STEPS - 1 {
                continue;
            }

            let area = Area {
                top: xy.y - marker_height / 2,
                left: xy.x - marker_width / 2,
                width: marker_width,
                height: marker_height,
            };

            let color = area.average_color(img);
            areas.insert(area, color);
        }

        Ok(Background { areas })
    }

    pub fn check_color(&self, xy: &XY) -> YUV {
        let mut y = 0.0;
        let mut u = 0.0;
        let mut v = 0.0;
        let mut distances = 0.0;

        for (area, color) in self.areas.iter() {
            // mult to bias towards closer points
            let distance = 1.0 / (xy.distance(&area.center()).powi(2));
            y += distance * color.y;
            u += distance * color.u;
            v += distance * color.v;
            distances += distance;
        }

        YUV {
            y: y / distances,
            u: u / distances,
            v: v / distances,
        }
    }

    pub fn areas(&self) -> &HashMap<Area, YUV> {
        &self.areas
    }
}

enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub fn flood_fill<FM>(img: &RgbaImage, xy: XY, match_color: FM) -> HashSet<XY>
where
    FM: Fn(&XY, &YUV) -> bool,
{
    let mut pixels = HashSet::new();
    let mut queue = vec![xy];

    loop {
        let Some(xy) = queue.pop() else {
            break;
        };

        if pixels.contains(&xy) {
            continue;
        }

        let pixel = img.get_pixel(xy.x, xy.y);
        let yuv = YUV::from_rgb(&pixel.to_rgb());

        if !match_color(&xy, &yuv) {
            continue;
        }

        pixels.insert(xy.clone());

        if xy.x > 0 {
            queue.push(XY {
                x: xy.x - 1,
                y: xy.y,
            });
        }

        if xy.y > 0 {
            queue.push(XY {
                x: xy.x,
                y: xy.y - 1,
            });
        }

        if xy.x < img.width() - 1 {
            queue.push(XY {
                x: xy.x + 1,
                y: xy.y,
            });
        }

        if xy.y < img.height() - 1 {
            queue.push(XY {
                x: xy.x,
                y: xy.y + 1,
            });
        }
    }

    pixels
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct XY {
    x: u32,
    y: u32,
}

impl XY {
    pub fn new(x: u32, y: u32) -> Self {
        XY { x, y }
    }

    fn distance(&self, other: &XY) -> f32 {
        let pow1 = (self.x as f32 - other.x as f32).powi(2);
        let pow2 = (self.y as f32 - other.y as f32).powi(2);
        (pow1 + pow2).sqrt()
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }
}

#[derive(Debug, Clone)]
pub struct YUV {
    y: f32,
    u: f32,
    v: f32,
}

impl YUV {
    fn from_rgb(pixel: &Rgb<u8>) -> YUV {
        let channels = pixel.channels();
        let r = channels[0] as f32 / 256.0;
        let g = channels[1] as f32 / 256.0;
        let b = channels[2] as f32 / 256.0;
        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        YUV {
            y,
            u: 0.492 * (b - y),
            v: 0.877 * (r - y),
        }
    }

    pub fn similar(&self, other: &Self, epsilon_y: f32, epsilon_uv: f32) -> bool {
        if (self.y - other.y).abs() > epsilon_y * 1.0 {
            return false;
        }

        if (self.u - other.u).abs() > epsilon_uv * 0.436 {
            return false;
        }

        if (self.v - other.v) > epsilon_uv * 0.615 {
            return false;
        }

        true
    }

    pub fn rgb(&self) -> [u8; 3] {
        let r = self.y + 1.14 * self.v;
        let g = self.y - 0.395 * self.u * 0.581 * self.v;
        let b = self.y + 2.033 * self.u;
        [(r * 256.0) as u8, (g * 256.0) as u8, (b * 256.0) as u8]
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

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Area {
    top: u32,
    left: u32,
    width: u32,
    height: u32,
}

impl Area {
    fn from_pixels(pixels: HashSet<XY>) -> Option<Area> {
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

    pub fn center(&self) -> XY {
        XY {
            x: self.left + self.width / 2,
            y: self.top + self.height / 2,
        }
    }

    pub fn contains(&self, xy: &XY) -> bool {
        xy.x >= self.left && xy.x <= self.right() && xy.y >= self.top && xy.y <= self.bottom()
    }

    pub fn color(&self, img: &mut RgbaImage, color: &[u8; 3]) {
        for x in self.left..self.right() {
            for y in self.top..self.bottom() {
                img.put_pixel(x, y, Rgb(*color).to_rgba());
            }
        }
    }

    fn average_color(&self, img: &RgbaImage) -> YUV {
        let mut y: Option<f32> = None;
        let mut u: Option<f32> = None;
        let mut v: Option<f32> = None;

        for px in self.left..self.right() {
            for py in self.top..self.bottom() {
                let pixel = img.get_pixel(px, py);
                let yuv = YUV::from_rgb(&pixel.to_rgb());

                y = match y {
                    Some(y) => Some((y + yuv.y) / 2.0),
                    None => Some(yuv.y),
                };

                u = match u {
                    Some(u) => Some((u + yuv.u) / 2.0),
                    None => Some(yuv.u),
                };

                v = match v {
                    Some(v) => Some((v + yuv.v) / 2.0),
                    None => Some(yuv.v),
                };
            }
        }

        YUV {
            y: y.unwrap(),
            u: u.unwrap(),
            v: v.unwrap(),
        }
    }

    pub fn top(&self) -> u32 {
        self.top
    }

    pub fn left(&self) -> u32 {
        self.left
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

struct EdgeIterator {
    a: XY,
    b: XY,
    steps: usize,
    next_step: usize,
}

impl EdgeIterator {
    pub fn new(a: XY, b: XY, steps: usize) -> Result<Self> {
        if steps < 2 {
            return Err(anyhow!(
                "requesting fewer than two steps seems a bit nonsensical"
            ));
        }

        Ok(Self {
            a,
            b,
            steps,
            next_step: 0,
        })
    }
}

impl Iterator for EdgeIterator {
    type Item = (usize, XY);

    fn next(&mut self) -> Option<Self::Item> {
        let current_step = self.next_step;
        if current_step >= self.steps {
            return None;
        }

        self.next_step += 1;

        let fraction = current_step as f32 / ((self.steps - 1) as f32);
        let length_x = self.b.x as f32 - self.a.x as f32;
        let length_y = self.b.y as f32 - self.a.y as f32;
        let x = self.a.x as f32 + fraction * length_x;
        let y = self.a.y as f32 + fraction * length_y;

        Some((
            current_step,
            XY {
                x: x as u32,
                y: y as u32,
            },
        ))
    }
}

pub struct IdentifiedSticker {
    pub area: Area,
    pub column: usize,
    pub row: usize,
}

pub struct IdentifiedStickers {
    stickers: Vec<IdentifiedSticker>,
}

impl IdentifiedStickers {
    pub fn new(img: &RgbaImage) -> Self {
        let mut areas: Vec<Area> = vec![];

        for ix in 0..img.width() {
            for iy in 0..img.height() {
                let xy = XY::new(ix, iy);

                let area_with_this_pixel_exists = areas.iter().any(|v| v.contains(&xy));
                if area_with_this_pixel_exists {
                    continue;
                }

                let color = img.get_pixel(xy.x(), xy.y());
                if color.to_rgba() == TRANSPARENT {
                    continue;
                }

                let pixels = flood_fill(img, xy, |xy: &XY, _yuv: &YUV| {
                    let color = img.get_pixel(xy.x(), xy.y());
                    color.to_rgba() != TRANSPARENT
                });

                let area = Area::from_pixels(pixels).unwrap();
                areas.push(area);
            }
        }

        areas.sort_by_key(|a| a.left());

        let snap_distance = img.width() as f32 * SNAP_STICKERS_THRESHOLD;

        let mut stickers_assigned_to_columns = vec![];
        for area in &areas {
            if stickers_assigned_to_columns.is_empty() {
                stickers_assigned_to_columns.push((area.clone(), 0));
            } else {
                let existing_column = stickers_assigned_to_columns
                    .iter()
                    .find(|v| {
                        (v.0.center().x as f32 - area.center().x as f32).abs() < snap_distance
                    })
                    .map(|v| v.1);
                match existing_column {
                    Some(column) => {
                        stickers_assigned_to_columns.push((area.clone(), column));
                    }
                    None => {
                        let highest_column = stickers_assigned_to_columns
                            .iter()
                            .map(|v| v.1)
                            .max()
                            .unwrap();
                        stickers_assigned_to_columns.push((area.clone(), highest_column + 1));
                    }
                }
            }
        }

        stickers_assigned_to_columns.sort_by(|a, b| match a.1.cmp(&b.1) {
            cmp::Ordering::Less => cmp::Ordering::Less,
            cmp::Ordering::Equal => a.0.top().partial_cmp(&b.0.top()).unwrap(),
            cmp::Ordering::Greater => cmp::Ordering::Greater,
        });

        let mut stickers: Vec<IdentifiedSticker> = vec![];
        let mut current_row = 0;
        for (area, column) in stickers_assigned_to_columns {
            match stickers.last() {
                Some(last) => {
                    if last.column != column {
                        current_row = 0;
                    } else {
                        current_row += 1;
                    }
                    stickers.push(IdentifiedSticker {
                        area,
                        column,
                        row: current_row,
                    });
                }
                None => stickers.push(IdentifiedSticker {
                    area,
                    column,
                    row: 0,
                }),
            }
        }

        Self { stickers }
    }

    pub fn stickers(&self) -> &[IdentifiedSticker] {
        &self.stickers
    }
}

pub fn is_at_least_this_much_of_image(pixels: usize, img: &RgbaImage, threshold: f32) -> bool {
    (pixels as f32) >= ((img.width() * img.height()) as f32 * threshold)
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
