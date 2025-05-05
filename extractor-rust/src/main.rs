#![feature(duration_constructors)]

use env_logger::Env;
use extractor_rust::{errors::Result, extractor::{flood_fill, Background, Markers, XY, YUV}};
use image::{ImageReader, Rgba};
use log::info;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Opening image...");
    let img = ImageReader::open("sticker_yellow.jpg")?.decode()?;
    let mut img = img.to_rgba8();

    info!("Locating markers...");
    let markers = Markers::find(&img)?;

    info!("Analysing background...");
    let background = Background::analyse(&img, &markers)?;

    info!("Coloring markers...");
    markers.top_left().color(&mut img, &[255, 0, 0]);
    markers.top_right().color(&mut img, &[255, 0, 0]);
    markers.bottom_left().color(&mut img, &[255, 0, 0]);
    markers.bottom_right().color(&mut img, &[255, 0, 0]);

    background
        .top_left()
        .color(&mut img, &background.top_left_color().rgb());
    background
        .top_right()
        .color(&mut img, &background.top_right_color().rgb());
    background
        .bottom_left()
        .color(&mut img, &background.bottom_left_color().rgb());
    background
        .bottom_right()
        .color(&mut img, &background.bottom_right_color().rgb());

    info!("Removing background...");
    let pixels = flood_fill(
        &img,
        background.top_left().left(),
        background.top_left().top(),
        &|xy: &XY, yuv: &YUV| {
            yuv.y() < 0.5 && yuv.u().abs() < 0.1 && yuv.v().abs() < 0.1
        },
    );
    for pixel in pixels {
        img.put_pixel(pixel.x(), pixel.y(), Rgba([0, 0, 0, 0]));
    }

    info!("Writing image...");
    img.save("empty.png")?;

    Ok(())
}
