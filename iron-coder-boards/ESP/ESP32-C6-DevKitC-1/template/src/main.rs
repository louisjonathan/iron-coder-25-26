#![no_std]
#![no_main]

use esp_hal::main;
use esp_println::println;

use common_hal_interface::*;

esp_bootloader_esp_idf::esp_app_desc!();
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[main]
fn main() -> ! {
    let peripherals = esp_with_serial_setup!();
    println!("Starting up...");

    loop {
        delay_ms!(100);
    }
}