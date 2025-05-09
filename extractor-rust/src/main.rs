use anyhow::Context;
use env_logger::Env;
use extractor_rust::{
    color::{Color, RGB},
    errors::Result,
    extractor::{
        Background, BackgroundDifference, IdentifiedStickers, Markers, TRANSPARENT, XY, flood_fill,
        is_at_least_this_much_of_image,
    },
};
use image::{ImageReader, Pixel, Rgb, RgbaImage, imageops::crop};
use log::info;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashSet, fs, path::Path, process::Command};
use tempfile::TempDir;

const INITIAL_CROP_FACTOR: f32 = 0.05; // 5%;

const BACKGROUND_DETECTION_FACTOR_L_POSITIVE: f32 = 0.30;
const BACKGROUND_DETECTION_FACTOR_L_NEGATIVE: f32 = 0.15;

const BACKGROUND_DETECTION_FACTOR_A_POSITIVE: f32 = 0.15;
const BACKGROUND_DETECTION_FACTOR_A_NEGATIVE: f32 = 0.15;

const BACKGROUND_DETECTION_FACTOR_B_POSITIVE: f32 = 0.30;
const BACKGROUND_DETECTION_FACTOR_B_NEGATIVE: f32 = 0.30;

// If a group of non-transparent pixels constitutes
// less than 2% of the image it will be made
// transparent.
const BACKGROUND_CLEANUP_FACTOR: f32 = 0.02;

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

    info!("Opening image {input_path}...");
    let img = ImageReader::open(input_path)?.decode()?;
    let mut img = img.to_rgba8();

    info!("Locating markers...");
    let markers = Markers::find(&img)?;

    // color markers for preview
    let red: Color = RGB::new(255, 0, 0).into();
    for marker in markers.markers() {
        marker.color(&mut img, &red);
    }

    info!("Analysing background...");
    let background = Background::analyse(&img, &markers)?;

    info!("Calculating background difference...");
    let background_difference = BackgroundDifference::new(&img, &background)?;

    // generate background measurements preview
    let mut preview_img = img.clone();
    for x in 0..preview_img.width() {
        for y in 0..preview_img.height() {
            let xy = XY::new(x, y);
            let color = background.check_color(&xy);
            let color: RGB = color.rgb();
            preview_img.put_pixel(x, y, Rgb([color.r(), color.g(), color.b()]).to_rgba());
        }
    }

    // color background measurement points in the preview and in the actual image
    for (area, color) in background.areas().iter() {
        area.color(&mut preview_img, color);
        area.color(&mut img, color);
    }

    preview.save(&img, "markers_and_background_measurements")?;
    preview.save(&preview_img, "interpolated_background")?;

    //let mut preview_img = img.clone();
    //for x in 0..preview_img.width() {
    //    for y in 0..preview_img.height() {
    //        let xy = XY::new(x, y);
    //        let distance = background_difference.get(&xy);

    //        //let color = LAB::new(80.0, distance.diff_l * 120.0, 0.0)?;
    //        //let color: Color = color.into();
    //        //let rgb = color.rgb();
    //        //preview_img.put_pixel(x, y, Rgb([rgb.r(), rgb.g(), rgb.b()]).to_rgba());

    //        let color = ((1.0 + distance.diff_l) / 2.0 * 255.0) as u8;
    //        preview_img.put_pixel(x, y, Rgb([color, color, color]).to_rgba());
    //    }
    //}
    //preview.save(&preview_img, "background_distance_l")?;

    //let mut preview_img = img.clone();
    //for x in 0..preview_img.width() {
    //    for y in 0..preview_img.height() {
    //        let xy = XY::new(x, y);
    //        let distance = background_difference.get(&xy);

    //        //let color = LAB::new(80.0, distance.diff_a * 120.0, 0.0)?;
    //        //let color: Color = color.into();
    //        //let rgb = color.rgb();
    //        //preview_img.put_pixel(x, y, Rgb([rgb.r(), rgb.g(), rgb.b()]).to_rgba());

    //        let color = ((1.0 + distance.diff_a) / 2.0 * 255.0) as u8;
    //        preview_img.put_pixel(x, y, Rgb([color, color, color]).to_rgba());
    //    }
    //}
    //preview.save(&preview_img, "background_distance_a")?;

    //let mut preview_img = img.clone();
    //for x in 0..preview_img.width() {
    //    for y in 0..preview_img.height() {
    //        let xy = XY::new(x, y);
    //        let distance = background_difference.get(&xy);

    //        //let color = LAB::new(80.0, distance.diff_b * 120.0, 0.0)?;
    //        //let color: Color = color.into();
    //        //let rgb = color.rgb();
    //        //preview_img.put_pixel(x, y, Rgb([rgb.r(), rgb.g(), rgb.b()]).to_rgba());

    //        let color = ((1.0 + distance.diff_b) / 2.0 * 255.0) as u8;
    //        preview_img.put_pixel(x, y, Rgb([color, color, color]).to_rgba());
    //    }
    //}
    //preview.save(&preview_img, "background_distance_b")?;

    info!("Removing background...");
    let pixels = flood_fill(
        &img,
        markers.middle_of_top_edge(),
        |xy: &XY, _color: &Color| {
            let difference = background_difference.get(xy);

            if difference.diff_l > 0.0
                && difference.diff_l.abs() > BACKGROUND_DETECTION_FACTOR_L_POSITIVE
            {
                return false;
            }

            if difference.diff_l < 0.0
                && difference.diff_l.abs() > BACKGROUND_DETECTION_FACTOR_L_NEGATIVE
            {
                return false;
            }

            if difference.diff_a > 0.0
                && difference.diff_a.abs() > BACKGROUND_DETECTION_FACTOR_A_POSITIVE
            {
                return false;
            }

            if difference.diff_a < 0.0
                && difference.diff_a.abs() > BACKGROUND_DETECTION_FACTOR_A_NEGATIVE
            {
                return false;
            }

            if difference.diff_b > 0.0
                && difference.diff_b.abs() > BACKGROUND_DETECTION_FACTOR_B_POSITIVE
            {
                return false;
            }

            if difference.diff_b < 0.0
                && difference.diff_b.abs() > BACKGROUND_DETECTION_FACTOR_B_NEGATIVE
            {
                return false;
            }

            true
        },
    );
    for pixel in pixels {
        img.put_pixel(pixel.x(), pixel.y(), TRANSPARENT);
    }

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

    preview.save(&img, "corrected_perspective")?;

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

    preview.save(&img, "initial_crop")?;

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

            let pixels = flood_fill(&img, xy, |xy: &XY, _color: &Color| {
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

    preview.save(&img, "background_cleanup")?;

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

    fn save(&mut self, img: &RgbaImage, name: &str) -> Result<()> {
        if self.save_intermediate_images {
            info!("Writing preview image...");
            img.save(format!(
                "{}_stage{}_{}.png",
                self.stem, self.stage_number, name
            ))?;
            self.stage_number += 1;
        }
        Ok(())
    }
}
