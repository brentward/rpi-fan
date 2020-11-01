use std::io::prelude::*;
use std::time::Duration;
use std::fs::File;
use std::io::BufReader;
use std::thread::sleep;
use gpio_cdev::{Chip, LineRequestFlags};

#[derive(Debug)]
enum FanError {
    GpioError(gpio_cdev::Error),
    IoError(std::io::Error),
    ParseIntError(std::num::ParseIntError),
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
    let mut chip = Chip::new("/dev/gpiochip0")?;
    let line = 18u32;
    let max_temp = 40000u32;
    let handle = chip
        .get_line(line)?
        .request(LineRequestFlags::OUTPUT, 1, "rpi-fan")?;
    loop {
        let file = File::open("/sys/class/thermal/thermal_zone0/temp")?;
        let mut buf_reader = BufReader::new(file);
        let mut line = String::new();
        let _len = buf_reader.read_line(&mut line)?;
        let temp: u32 = line.trim().parse()?;
        if temp > max_temp {
            handle.set_value(1)?;
            sleep(Duration::from_secs(120));
        } else {
            handle.set_value(0)?;
            sleep(Duration::from_secs(30));
        }
    }
}
