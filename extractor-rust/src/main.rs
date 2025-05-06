#![feature(duration_constructors)]

use env_logger::Env;
use extractor_rust::{
    errors::Result,
    extractor::{Background, Markers, XY, YUV, flood_fill},
};
use image::{ImageReader, Pixel, Rgb, Rgba};
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

    //for x in background.top_left().left()..background.top_right().left() {
    //    for y in background.top_left().top()..background.bottom_right().top() {
    //        let expected_color = background.check_color(&XY::new(x, y));
    //            img.put_pixel(x, y, Rgb(expected_color.rgb()).to_rgba());
    //    }
    //}

    info!("Removing background...");
    let pixels = flood_fill(
        &img,
        background.top_left().left(),
        background.top_left().top(),
        &|xy: &XY, yuv: &YUV| {
            let expected_color = background.check_color(xy);
            //println!("expected={:?} encountered={:?}", expected_color, yuv);
            //println!("expected={:?}", expected_color);
            expected_color.similar(yuv, 0.1)
            //yuv.y() < 0.5 && yuv.u().abs() < 0.1 && yuv.v().abs() < 0.1 }
        },
    );
    for pixel in pixels {
        img.put_pixel(pixel.x(), pixel.y(), Rgba([0, 0, 0, 0]));
    }

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

    info!("Writing image...");
    img.save("empty.png")?;

    Ok(())
}
