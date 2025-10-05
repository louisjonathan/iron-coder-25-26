#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig, Input, InputConfig, Pull};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
esp_bootloader_esp_idf::esp_app_desc!();
fn blocking_delay(duration: Duration) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < duration {}
}
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let _peripherals = esp_hal::init(config);

    loop {
        blocking_delay(Duration::from_millis(100));
    }
}