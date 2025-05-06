#![feature(duration_constructors)]

use std::{collections::HashSet, process::Command};

use env_logger::Env;
use extractor_rust::{
    errors::Result,
    extractor::{Background, Markers, XY, YUV, flood_fill},
};
use image::{GenericImageView, ImageReader, Pixel, Rgba, imageops::crop};
use log::info;
use tempfile::TempDir;

const INITIAL_CROP_FACTOR: f32 = 0.1; // 10%;
const BACKGROUND_CLEANUP_FACTOR: f32 = 0.05; // If a group of non-transparent pixels constitutes
// less than 1% of the image it will be made
// transparent.
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Opening image...");
    let img = ImageReader::open("yellow.jpg")?.decode()?;
    let mut img = img.to_rgba8();

    info!("Writing preview image...");
    img.save("stage0.png")?;

    info!("Locating markers...");
    let markers = Markers::find(&img)?;

    info!("Analysing background...");
    let background = Background::analyse(&img, &markers)?;

    //for x in background.top_left().left()..background.top_right().left() {
    //    for y in background.top_left().top()..background.bottom_right().top() {
    //        let expected_color = background.check_color(&XY::new(x, y));
    //            img.put_pixel(x, y, Rgb(expected_color.rgb()).to_rgba());
    //    }
    //}

    info!("Removing background...");
    let pixels = flood_fill(&img, markers.middle_of_top_edge(), |xy: &XY, yuv: &YUV| {
        let expected_color = background.check_color(xy);
        //println!("expected={:?} encountered={:?}", expected_color, yuv);
        //println!("expected={:?}", expected_color);
        expected_color.similar(yuv, 0.15)
        //yuv.y() < 0.5 && yuv.u().abs() < 0.1 && yuv.v().abs() < 0.1 }
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
    let magick_input = tmp_dir.path().join("stage1.png");
    let magick_output = tmp_dir.path().join("stage2.png");

    info!("Writing preview image...");
    img.save("stage1.png")?;

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

    //background
    //    .top_left()
    //    .color(&mut img, &background.top_left_color().rgb());
    //background
    //    .top_right()
    //    .color(&mut img, &background.top_right_color().rgb());
    //background
    //    .bottom_left()
    //    .color(&mut img, &background.bottom_left_color().rgb());
    //background
    //    .bottom_right()
    //    .color(&mut img, &background.bottom_right_color().rgb());

    info!("Writing preview image...");
    img.save("stage2.png")?;

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

    info!("Writing preview image...");
    img.save("stage3.png")?;

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

            let pixels = flood_fill(&img, xy, |xy: &XY, yuv: &YUV| {
                let color = img.get_pixel(xy.x(), xy.y());
                color.to_rgba() != TRANSPARENT
            });

            if (pixels.len() as f32)
                < ((img.width() * img.height()) as f32 * BACKGROUND_CLEANUP_FACTOR)
            {
                for pixel in &pixels {
                    img.put_pixel(pixel.x(), pixel.y(), TRANSPARENT);
                }
            }

            for pixel in &pixels {
                skip.insert(pixel.clone());
            }
        }
    }

    info!("Writing preview image...");
    img.save("stage4.png")?;

    Ok(())
}
