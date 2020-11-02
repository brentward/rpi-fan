extern crate config;

use std::io::prelude::*;
use std::time::Duration;
use std::fs::File;
use std::io::BufReader;
use std::thread::sleep;
use gpio_cdev::{Chip, LineRequestFlags};
use config::{ConfigError, Config, File as ConfigFile};
use serde::Deserialize;

#[derive(Debug)]
enum FanError {
    GpioError(gpio_cdev::Error),
    IoError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
    ConfigError(ConfigError),
}

impl From<gpio_cdev::Error> for FanError {
    fn from (error: gpio_cdev::Error) -> Self {
        FanError::GpioError(error)
    }
}

impl From<std::io::Error> for FanError {
    fn from (error: std::io::Error) -> Self {
        FanError::IoError(error)
    }
}

impl From<std::num::ParseIntError> for FanError {
    fn from (error: std::num::ParseIntError) -> Self {
        FanError::ParseIntError(error)
    }
}

impl From<ConfigError> for FanError {
    fn from (error: ConfigError) -> Self {
        FanError::ConfigError(error)
    }
}

#[derive(Deserialize, Debug)]
struct Settings {
    gpio_path: String,
    gpio_pin: u32,
    temp_max: u32,
    temp_path: String,
    retry_interval_hot: u64,
    retry_interval_cool: u64,
}

impl Settings {
    fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();
        settings.set_default("gpio_path", String::from("/dev/gpiochip0"))?;
        settings.set_default("gpio_pin", 18)?;
        settings.set_default("temp_max", 40)?;
        settings.set_default("temp_path", String::from("/sys/class/thermal/thermal_zone0/temp"))?;
        settings.set_default("retry_interval_hot", 30)?;
        settings.set_default("retry_interval_cool", 120)?;
        settings.merge(ConfigFile::with_name("/etc/default/rpi-fan").required(false))?;

        settings.try_into()
    }
}
fn main() {
    std::process::exit(match run_fan() {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            1
        }
    });
}

fn run_fan() -> std::result::Result<(), FanError> {
    let settings = Settings::new()?;
    let mut chip = Chip::new(settings.gpio_path)?;
    let handle = chip
        .get_line(settings.gpio_pin)?
        .request(LineRequestFlags::OUTPUT, 1, "rpi-fan")?;
    loop {
        let file = File::open(settings.temp_path.as_str())?;
        let mut buf_reader = BufReader::new(file);
        let mut line = String::new();
        let _len = buf_reader.read_line(&mut line)?;
        let temp: u32 = line.trim().parse()?;
        if temp > (settings.temp_max * 1000) {
            handle.set_value(1)?;
            sleep(Duration::from_secs(settings.retry_interval_hot));
        } else {
            handle.set_value(0)?;
            sleep(Duration::from_secs(settings.retry_interval_cool));
        }
    }
}
