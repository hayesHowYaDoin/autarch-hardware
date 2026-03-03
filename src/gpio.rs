use std::{sync::mpsc::Sender, time::Duration};

use color_eyre::eyre::Result;

use crate::event::KeyEvent;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GpioPin(pub u8);

pub type EventCallback = Box<dyn Fn() + Send>;

/// Abstraction over GPIO input pins with interrupt support.
/// Allows swapping between real hardware (rppal) and mock implementations.
pub trait GpioInput: Send {
    fn set_callbacks(
        &mut self,
        debounce: Duration,
        on_press: EventCallback,
        on_release: EventCallback,
    ) -> Result<()>;
}

#[cfg(target_arch = "aarch64")]
pub mod real {
    use super::*;
    use rppal::gpio::{Gpio, InputPin, Trigger};

    pub struct RppalInput {
        pin: InputPin,
    }

    impl RppalInput {
        pub fn new(gpio_num: u8) -> Result<Self> {
            let pin = Gpio::new()?.get(gpio_num)?.into_input_pullup();
            Ok(Self { pin })
        }
    }

    impl GpioInput for RppalInput {
        fn set_callbacks(
            &mut self,
            debounce: Duration,
            on_press: EventCallback,
            on_release: EventCallback,
        ) -> Result<()> {
            self.pin
                .set_async_interrupt(Trigger::Both, Some(debounce), move |event| {
                    match event.trigger {
                        Trigger::FallingEdge => on_press(),
                        Trigger::RisingEdge => on_release(),
                        _ => {}
                    }
                })?;
            Ok(())
        }
    }
}

#[cfg(not(target_arch = "aarch64"))]
pub mod mock {
    use super::*;
    use crossterm::event::{self, Event, KeyCode, KeyModifiers};
    use crossterm::terminal;
    use log::info;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex, OnceLock};
    use std::thread;

    type CallbackMap = Arc<Mutex<HashMap<u8, (Option<EventCallback>, Option<EventCallback>)>>>;

    static CALLBACKS: OnceLock<CallbackMap> = OnceLock::new();
    static LISTENER_STARTED: AtomicBool = AtomicBool::new(false);

    fn get_callbacks() -> &'static CallbackMap {
        CALLBACKS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
    }

    pub struct MockInput {
        gpio_num: u8,
    }

    impl MockInput {
        pub fn new(gpio_num: u8) -> Result<Self> {
            info!(
                "[MOCK] GPIO {} initialized (press '{}' for press, Shift+'{}' for release)",
                gpio_num, gpio_num, gpio_num
            );
            Ok(Self { gpio_num })
        }
    }

    pub fn start_keyboard_listener() -> Result<()> {
        if LISTENER_STARTED.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let callbacks = Arc::clone(get_callbacks());

        terminal::enable_raw_mode()?;
        info!("[MOCK] Keyboard listener started. Number=press, Shift+Number (!@#...)=release, Esc=quit.");

        thread::spawn(move || {
            let _guard = RawModeGuard;

            loop {
                if let Ok(Event::Key(key_event)) = event::read() {
                    if key_event.code == KeyCode::Esc
                        || (key_event.modifiers.contains(KeyModifiers::CONTROL)
                            && key_event.code == KeyCode::Char('c'))
                    {
                        info!("[MOCK] Exiting");
                        let _ = terminal::disable_raw_mode();
                        std::process::exit(0);
                    }

                    if let KeyCode::Char(c) = key_event.code {
                        // Map digits to (gpio_num, is_release)
                        // Shift+number gives symbols: !@#$%^&*()
                        let gpio_event = match c {
                            '0'..='9' => Some((c.to_digit(10).unwrap() as u8, false)),
                            ')' => Some((0, true)),
                            '!' => Some((1, true)),
                            '@' => Some((2, true)),
                            '#' => Some((3, true)),
                            '$' => Some((4, true)),
                            '%' => Some((5, true)),
                            '^' => Some((6, true)),
                            '&' => Some((7, true)),
                            '*' => Some((8, true)),
                            '(' => Some((9, true)),
                            _ => None,
                        };

                        if let Some((gpio_num, is_release)) = gpio_event {
                            let cbs = callbacks.lock().unwrap();
                            if let Some((press_cb, release_cb)) = cbs.get(&gpio_num) {
                                if is_release {
                                    info!("[MOCK] GPIO {} release", gpio_num);
                                    if let Some(ref cb) = release_cb {
                                        cb();
                                    }
                                } else {
                                    info!("[MOCK] GPIO {} press", gpio_num);
                                    if let Some(ref cb) = press_cb {
                                        cb();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    struct RawModeGuard;

    impl Drop for RawModeGuard {
        fn drop(&mut self) {
            let _ = terminal::disable_raw_mode();
        }
    }

    impl GpioInput for MockInput {
        fn set_callbacks(
            &mut self,
            _debounce: Duration,
            on_press: EventCallback,
            on_release: EventCallback,
        ) -> Result<()> {
            let mut cbs = get_callbacks().lock().unwrap();
            cbs.insert(self.gpio_num, (Some(on_press), Some(on_release)));
            Ok(())
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub use real::RppalInput as PlatformInput;

#[cfg(not(target_arch = "aarch64"))]
pub use mock::MockInput as PlatformInput;

#[cfg(not(target_arch = "aarch64"))]
use mock::start_keyboard_listener;

#[cfg(target_arch = "aarch64")]
fn start_keyboard_listener() -> Result<()> {
    Ok(())
}

pub fn initialize(pins: &[GpioPin], tx: Sender<KeyEvent>) -> Result<Vec<Box<dyn GpioInput>>> {
    let mut inputs: Vec<Box<dyn GpioInput>> = Vec::new();

    for &pin in pins {
        let pin_num = pin.0;
        let mut input = PlatformInput::new(pin_num)?;

        let tx_press = tx.clone();
        let tx_release = tx.clone();
        input.set_callbacks(
            Duration::from_millis(50),
            Box::new(move || {
                let _ = tx_press.send(KeyEvent::Press(pin_num));
            }),
            Box::new(move || {
                let _ = tx_release.send(KeyEvent::Release(pin_num));
            }),
        )?;

        inputs.push(Box::new(input));
    }

    start_keyboard_listener()?;

    Ok(inputs)
}
