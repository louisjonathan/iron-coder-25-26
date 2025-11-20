#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use common_hal_interface::*;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[main]
fn main() -> ! {
    let peripherals = esp_with_serial_setup!();
    let delay = Delay::new();
    println!("Starting up...");

    // PIN_DEFINITIONS

    loop {
        delay.delay(Duration::from_millis(1000));
    }
}
