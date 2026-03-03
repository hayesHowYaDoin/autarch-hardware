use std::{collections::HashMap, sync::mpsc::Receiver};

use color_eyre::Result;
use enigo::{Direction, Enigo, Key, Keyboard};
use log::debug;

use crate::gpio::GpioPin;

#[derive(Clone, Copy, Debug)]
pub enum KeyEvent {
    Press(u8),
    Release(u8),
}

pub fn process_events(
    rx: Receiver<KeyEvent>,
    keymap: &HashMap<GpioPin, Key>,
    mut enigo: Enigo,
) -> Result<()> {
    for event in rx {
        debug!("Received event: {:?}", event);
        let (gpio_num, direction) = match event {
            KeyEvent::Press(pin) => (pin, Direction::Press),
            KeyEvent::Release(pin) => (pin, Direction::Release),
        };

        if let Some(&key) = keymap.get(&GpioPin(gpio_num)) {
            enigo.key(key, direction)?;
        }
    }
    Ok(())
}
