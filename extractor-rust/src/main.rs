#![feature(duration_constructors)]

use anyhow::Context;
use env_logger::Env;
use extractor_rust::{
    color::{Color, RGB},
    errors::Result,
    extractor::{
        AverageColors, Background, Gradient, IdentifiedStickers, Markers, TRANSPARENT, XY,
        flood_fill, is_at_least_this_much_of_image,
    },
};
use image::{ImageReader, Pixel, Rgb, RgbaImage, imageops::crop};
use log::info;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashSet, fs, path::Path, process::Command};
use tempfile::TempDir;

const INITIAL_CROP_FACTOR: f32 = 0.05; // 5%;

// If a group of non-transparent pixels constitutes
// less than 2% of the image it will be made
// transparent.
const BACKGROUND_CLEANUP_FACTOR: f32 = 0.02;

// 0.5%; fraction of image width.
const LARGER_GRADIENT_PIXEL_FACTOR: f32 = 0.005;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let command = clap::Command::new("extractor")
        .about("A program which processes photos of stickers")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            clap::Command::new("debug")
                .about("Debug the extraction process")
                .arg(clap::arg!(<INPUT_FILE> "The input file to process"))
                .arg_required_else_help(true),
        )
        .subcommand(
            clap::Command::new("extract")
                .about("Run the extraction process for a directory")
                .arg(clap::arg!(<SOURCE_DIRECTORY> "The source directory"))
                .arg(clap::arg!(<TARGET_DIRECTORY> "The target directory")),
        );

    let matches = command.get_matches();

    match matches.subcommand() {
        Some(("debug", sub_matches)) => {
            let file_path = sub_matches.get_one::<String>("INPUT_FILE").unwrap();
            extract(file_path, "./", true)?;
            Ok(())
        }
        Some(("extract", sub_matches)) => {
            let source_directory = sub_matches.get_one::<String>("SOURCE_DIRECTORY").unwrap();
            let target_directory = sub_matches.get_one::<String>("TARGET_DIRECTORY").unwrap();

            let readdir =
                fs::read_dir(source_directory).context("error listing the source directory")?;
            let mut paths: Vec<String> = vec![];
            for v in readdir {
                paths.push(v?.path().to_string_lossy().to_string());
            }

            paths.par_iter().for_each(|file_path| {
                extract(file_path, target_directory, false).unwrap();
            });

            Ok(())
        }
        _ => unreachable!(),
    }
}

fn extract(input_path: &str, output_directory: &str, save_intermediate_images: bool) -> Result<()> {
    let mut preview = PreviewImagesSaver::new(input_path, save_intermediate_images)?;

    info!("Opening image {}...", input_path);
    let img = ImageReader::open(input_path)?.decode()?;
    let mut img = img.to_rgba8();

    info!("Locating markers...");
    let markers = Markers::find(&img)?;

    info!("Coloring markers...");
    let red: Color = RGB::new(255, 0, 0).into();
    for marker in markers.markers() {
        marker.color(&mut img, &red);
    }

    preview.save(&img)?;

    info!("Generating average colors...");
    let gradient_size = 1;
    let average_colors = AverageColors::new(&img, gradient_size)?;

    let mut average_colors_img = img.clone();
    for x in 0..average_colors_img.width() {
        for y in 0..average_colors_img.height() {
            let xy = XY::new(x, y);
            let color = average_colors.average_color(&xy);
            let rgb = color.rgb();
            average_colors_img.put_pixel(x, y, Rgb([rgb.r(), rgb.g(), rgb.b()]).to_rgba());
        }
    }
    preview.save(&average_colors_img)?;

    info!("Generating a gradient...");
    let gradient = Gradient::new(&img, &average_colors)?;

    let mut gradient_img = img.clone();
    for x in 0..gradient_img.width() {
        for y in 0..gradient_img.height() {
            let xy = XY::new(x, y);
            let gradient = gradient.get_gradient(&xy);
            let color = (gradient * 255.0) as u8;
            gradient_img.put_pixel(x, y, Rgb([color, color, color]).to_rgba());
            //gradient_img.put_pixel(x, y, Rgb([gradient, gradient, gradient]).to_rgba());
        }
    }
    preview.save(&gradient_img)?;

    info!("Generating large average colors...");
    let gradient_size = (img.width() as f32 * LARGER_GRADIENT_PIXEL_FACTOR) as u32;
    let large_average_colors = AverageColors::new(&img, gradient_size)?;

    let mut large_average_colors_img = img.clone();
    for x in 0..large_average_colors_img.width() {
        for y in 0..large_average_colors_img.height() {
            let xy = XY::new(x, y);
            let color = large_average_colors.average_color(&xy);
            let rgb = color.rgb();
            large_average_colors_img.put_pixel(x, y, Rgb([rgb.r(), rgb.g(), rgb.b()]).to_rgba());
        }
    }
    preview.save(&large_average_colors_img)?;

    info!("Generating a large gradient...");
    let large_gradient = Gradient::new(&img, &large_average_colors)?;

    let mut large_gradient_img = img.clone();
    for x in 0..large_gradient_img.width() {
        for y in 0..large_gradient_img.height() {
            let xy = XY::new(x, y);
            let gradient = large_gradient.get_gradient(&xy);
            let color = (gradient * 255.0) as u8;
            large_gradient_img.put_pixel(x, y, Rgb([color, color, color]).to_rgba());
            //gradient_img.put_pixel(x, y, Rgb([gradient, gradient, gradient]).to_rgba());
        }
    }
    preview.save(&large_gradient_img)?;

    info!("Analysing background...");
    let background = Background::analyse(&img, &markers)?;

    info!("Removing background...");
    let pixels = flood_fill(
        &img,
        markers.middle_of_top_edge(),
        |xy: &XY, _color: &Color| {
            let gradient = gradient.get_gradient(xy);
            let large_gradient = large_gradient.get_gradient(xy);
            //let gradient_color: LAB = gradient_color.lab();

            //let distance = (gradient_color.y().powi(4)
            //    + gradient_color.u().powi(2)
            //    + gradient_color.v().powi(2))
            //.sqrt();
            gradient < 0.05 && large_gradient < 0.1

            //if gradient_color.y() > 0.1 {
            //    return false;
            //}

            //if gradient_color.u() > 0.02 {
            //    return false;
            //}

            //if gradient_color.v() > 0.05 {
            //    return false;
            //}

            //true

            //let expected_color = background.check_color(xy);
            //expected_color.similar(
            //    yuv,
            //    BACKGROUND_SIMILARITY_FACTOR_Y,
            //    BACKGROUND_SIMILARITY_FACTOR_UV,
            //)
        },
    );
    for pixel in pixels {
        img.put_pixel(pixel.x(), pixel.y(), TRANSPARENT);
    }

    info!("Coloring background measurements...");
    for (area, color) in background.areas().iter() {
        area.color(&mut img, color);
    }

    preview.save(&img)?;

    info!("Correcting perspective...");
    let tmp_dir = TempDir::new()?;
    let magick_input = tmp_dir.path().join("input.png");
    let magick_output = tmp_dir.path().join("output.png");

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

    preview.save(&img)?;

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

    preview.save(&img)?;

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

            let pixels = flood_fill(&img, xy, |xy: &XY, color: &Color| {
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

    preview.save(&img)?;

    info!("Final crop...");
    let path = Path::new(&input_path);
    let file_stem = path.file_stem().unwrap();

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

struct PreviewImagesSaver {
    stem: String,
    save_intermediate_images: bool,
    stage_number: u32,
}

impl PreviewImagesSaver {
    fn new(input_path: impl Into<String>, save_intermediate_images: bool) -> Result<Self> {
        let input_path: String = input_path.into();
        let path = Path::new(&input_path);
        let stem = path.file_stem().unwrap();
        Ok(Self {
            stem: stem.to_str().unwrap().into(),
            save_intermediate_images,
            stage_number: 0,
        })
    }

    fn save(&mut self, img: &RgbaImage) -> Result<()> {
        if self.save_intermediate_images {
            info!("Writing preview image...");
            img.save(format!("{}_stage{}.png", self.stem, self.stage_number))?;
            self.stage_number += 1;
        }
        Ok(())
    }
}
