use std::collections::HashMap;

use clap::Parser;
use enigo::Key;

use crate::gpio::GpioPin;

fn parse_key(s: &str) -> Result<Key, String> {
    match s.to_lowercase().as_str() {
        "space" => Ok(Key::Space),
        "return" | "enter" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "shift" => Ok(Key::Shift),
        "control" | "ctrl" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        other => {
            let chars: Vec<char> = other.chars().collect();
            if chars.len() == 1 {
                Ok(Key::Unicode(chars[0]))
            } else {
                Err(format!("Unknown key: {}", s))
            }
        }
    }
}

fn parse_key_val(s: &str) -> Result<(GpioPin, Key), String> {
    let (gpio_str, key_str) = s
        .split_once(':')
        .ok_or_else(|| format!("Invalid format '{}', expected GPIO:KEY", s))?;

    let gpio_num: u8 = gpio_str
        .parse()
        .map_err(|_| format!("Invalid GPIO number: {}", gpio_str))?;

    let key = parse_key(key_str)?;

    Ok((GpioPin(gpio_num), key))
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// GPIO-to-key assignments
    #[arg(long, value_parser = parse_key_val, value_delimiter = ' ', num_args = 1..)]
    pub assignments: Vec<(GpioPin, Key)>,
}

pub fn parse_arguments() -> HashMap<GpioPin, Key> {
    let args = Args::parse();
    args.assignments.into_iter().collect()
}
