#![feature(duration_constructors)]

use std::{collections::HashSet, process::Command};

use env_logger::Env;
use extractor_rust::{
    errors::Result,
    extractor::{Background, Markers, XY, YUV, flood_fill, is_at_least_this_much_of_image},
};
use image::{ImageReader, Pixel, Rgba, imageops::crop};
use log::info;
use tempfile::TempDir;

const INITIAL_CROP_FACTOR: f32 = 0.1; // 10%;

// If a group of non-transparent pixels constitutes
// less than 1% of the image it will be made
// transparent.
const BACKGROUND_CLEANUP_FACTOR: f32 = 0.05;

const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    extract("yellow_day.jpg", "yellow_day_out.png", false)?;
    extract("black_day.jpg", "black_day_out.png", false)?;
    extract("yellow.jpg", "yellow_out.png", false)?;
    extract("black.jpg", "black_out.png", false)?;

    Ok(())
}

fn extract(input_path: &str, output_path: &str, save_intermediate_images: bool) -> Result<()> {
    info!("Opening image {}...", input_path);

    let img = ImageReader::open(input_path)?.decode()?;
    let mut img = img.to_rgba8();

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save("stage0.png")?;
    }

    info!("Locating markers...");
    let markers = Markers::find(&img)?;

    info!("Coloring markers...");
    for marker in markers.markers() {
        marker.color(&mut img, &[255, 0, 0]);
    }

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save("stage1.png")?;
    }

    info!("Analysing background...");
    let background = Background::analyse(&img, &markers)?;

    info!("Removing background...");
    let pixels = flood_fill(&img, markers.middle_of_top_edge(), |xy: &XY, yuv: &YUV| {
        let expected_color = background.check_color(xy);
        expected_color.similar(yuv, 0.15)
    });
    for pixel in pixels {
        img.put_pixel(pixel.x(), pixel.y(), TRANSPARENT);
    }

    info!("Coloring markers...");
    for marker in markers.markers() {
        marker.color(&mut img, &[255, 0, 0]);
    }

    for (area, color) in background.areas().iter() {
        area.color(&mut img, &color.rgb());
    }

    info!("Correcting perspective...");
    let tmp_dir = TempDir::new()?;
    let magick_input = tmp_dir.path().join("input.png");
    let magick_output = tmp_dir.path().join("output.png");

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save("stage2.png")?;
    }

    info!("Writing image...");
    img.save(&magick_input)?;

    let perspective_params = format!(
        "{},{} {},{} {},{} {},{} {},{} {},{} {},{} {},{}",
        markers.top_left().center().x(),
        markers.top_left().center().y(),
        0,
        0,
        markers.top_right().center().x(),
        markers.top_right().center().y(),
        img.width(),
        0,
        markers.bottom_left().center().x(),
        markers.bottom_left().center().y(),
        0,
        img.height(),
        markers.bottom_right().center().x(),
        markers.bottom_right().center().y(),
        img.width(),
        img.height(),
    );

    Command::new("magick")
        .arg(&magick_input)
        .arg("-alpha")
        .arg("set")
        .arg("-virtual-pixel")
        .arg("transparent")
        .arg("-distort")
        .arg("Perspective")
        .arg(perspective_params)
        .arg(&magick_output)
        .output()?;

    let img = ImageReader::open(magick_output)?.decode()?;
    let mut img = img.to_rgba8();

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save("stage3.png")?;
    }

    info!("Cropping...");
    let width = img.width();
    let height = img.height();

    let img = crop(
        &mut img,
        (width as f32 * INITIAL_CROP_FACTOR) as u32,
        (height as f32 * INITIAL_CROP_FACTOR) as u32,
        (width as f32 * (1.0 - 2.0 * INITIAL_CROP_FACTOR)) as u32,
        (height as f32 * (1.0 - 2.0 * INITIAL_CROP_FACTOR)) as u32,
    );
    let mut img = img.to_image();

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save("stage4.png")?;
    }

    info!("Cleaning up background...");
    let mut skip: HashSet<XY> = HashSet::new();

    for ix in 0..img.width() {
        for iy in 0..img.height() {
            let xy = XY::new(ix, iy);

            if skip.contains(&xy) {
                continue;
            }

            let color = img.get_pixel(xy.x(), xy.y());
            if color.to_rgba() == TRANSPARENT {
                continue;
            }

            let pixels = flood_fill(&img, xy, |xy: &XY, _yuv: &YUV| {
                let color = img.get_pixel(xy.x(), xy.y());
                color.to_rgba() != TRANSPARENT
            });

            if !is_at_least_this_much_of_image(pixels.len(), &img, BACKGROUND_CLEANUP_FACTOR) {
                for pixel in &pixels {
                    img.put_pixel(pixel.x(), pixel.y(), TRANSPARENT);
                }
            }

            for pixel in &pixels {
                skip.insert(pixel.clone());
            }
        }
    }

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save("stage5.png")?;
    }

    info!("Final crop...");
    let min_x = img
        .enumerate_pixels()
        .filter(|(_x, _y, color)| *color != &TRANSPARENT)
        .map(|(x, _y, _color)| x)
        .min()
        .unwrap();

    let max_x = img
        .enumerate_pixels()
        .filter(|(_x, _y, color)| *color != &TRANSPARENT)
        .map(|(x, _y, _color)| x)
        .max()
        .unwrap();

    let min_y = img
        .enumerate_pixels()
        .filter(|(_x, _y, color)| *color != &TRANSPARENT)
        .map(|(_x, y, _color)| y)
        .min()
        .unwrap();

    let max_y = img
        .enumerate_pixels()
        .filter(|(_x, _y, color)| *color != &TRANSPARENT)
        .map(|(_x, y, _color)| y)
        .max()
        .unwrap();

    let img = crop(&mut img, min_x, min_y, max_x - min_x, max_y - min_y);
    let img = img.to_image();

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save("stage6.png")?;
    }

    info!("Writing final image...");
    img.save(output_path)?;

    Ok(())
}
