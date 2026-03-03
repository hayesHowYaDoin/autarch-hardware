mod cli;
mod event;
mod gpio;

use color_eyre::eyre::Result;
use enigo::{Enigo, Settings};
use log::debug;
use simplelog::*;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::sync::mpsc::channel;

use crate::{cli::parse_arguments, event::process_events, gpio::GpioPin};

fn main() -> Result<()> {
    color_eyre::install()?;

    let log_path: PathBuf = env::var("AUTARCH_HARDWARE_LOG_PATH")
        .unwrap_or_else(|_| ".".to_string())
        .into();
    let log_file = log_path.join("autarch.log");

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create(log_file)?,
    )])?;

    let keymap = parse_arguments();
    debug!("Key Map: {:?}", keymap);

    let pins: Vec<GpioPin> = keymap.keys().copied().collect();

    let (tx, rx) = channel();
    let _inputs = gpio::initialize(&pins, tx)?;

    let enigo = Enigo::new(&Settings::default())?;
    process_events(rx, &keymap, enigo)?;

    Ok(())
}
