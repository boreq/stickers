#![feature(duration_constructors)]

use std::{collections::HashSet, env, fs, path::Path, process::Command};

use env_logger::Env;
use extractor_rust::{
    errors::Result,
    extractor::{
        Background, IdentifiedStickers, Markers, TRANSPARENT, XY, YUV, flood_fill,
        is_at_least_this_much_of_image,
    },
};
use image::{ImageReader, Pixel, imageops::crop};
use log::info;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tempfile::TempDir;

const INITIAL_CROP_FACTOR: f32 = 0.05; // 5%;

// If a group of non-transparent pixels constitutes
// less than 2% of the image it will be made
// transparent.
const BACKGROUND_CLEANUP_FACTOR: f32 = 0.02;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();

    let readdir = fs::read_dir(&args[1])?;
    let mut paths: Vec<String> = vec![];
    for v in readdir {
        paths.push(v?.path().to_string_lossy().to_string());
    }

    paths.par_iter().for_each(|input_path| {
        extract(input_path, &args[2], false).unwrap();
    });

    Ok(())
}

fn extract(input_path: &str, output_directory: &str, save_intermediate_images: bool) -> Result<()> {
    let path = Path::new(input_path);
    let file_stem = path.file_stem().unwrap();

    info!("Opening image {}...", input_path);

    let img = ImageReader::open(input_path)?.decode()?;
    let mut img = img.to_rgba8();

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save(format!("{}_stage0.png", file_stem.to_str().unwrap()))?;
    }

    info!("Locating markers...");
    let markers = Markers::find(&img)?;

    info!("Coloring markers...");
    for marker in markers.markers() {
        marker.color(&mut img, &[255, 0, 0]);
    }

    if save_intermediate_images {
        info!("Writing preview image...");
        img.save(format!("{}_stage1.png", file_stem.to_str().unwrap()))?;
    }

    info!("Analysing background...");
    let background = Background::analyse(&img, &markers)?;

    info!("Removing background...");
    let pixels = flood_fill(&img, markers.middle_of_top_edge(), |xy: &XY, yuv: &YUV| {
        let expected_color = background.check_color(xy);
        expected_color.similar(yuv, 0.2, 0.1)
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
        img.save(format!("{}_stage2.png", file_stem.to_str().unwrap()))?;
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
        img.save(format!("{}_stage3.png", file_stem.to_str().unwrap()))?;
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
        img.save(format!("{}_stage4.png", file_stem.to_str().unwrap()))?;
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
        img.save(format!("{}_stage5.png", file_stem.to_str().unwrap()))?;
    }

    info!("Final crop...");
    let stickers = IdentifiedStickers::new(&img);
    for sticker in stickers.stickers() {
        let img = crop(
            &mut img,
            sticker.area.left(),
            sticker.area.top(),
            sticker.area.width(),
            sticker.area.height(),
        );
        let img = img.to_image();

        let output_path = Path::new(output_directory).join(format!(
            "{}_{}_{}.png",
            file_stem.to_str().unwrap(),
            sticker.column,
            sticker.row
        ));

        info!("Writing final image...");
        img.save(output_path)?;
    }

    Ok(())
}
