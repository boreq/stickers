#![feature(duration_constructors)]

use env_logger::Env;
use extractor_rust::{errors::Result, extractor::extract};
use log::info;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("test");
    extract("sticker_yellow.jpg")?;
    Ok(())
}
