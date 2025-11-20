#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use common_hal_interface::*;
use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text},
};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_hal::{clock::CpuClock, time::Rate};
use esp_println::println;
use heapless::String;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

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
    let scl = peripherals.GPIO15;
    let sda = peripherals.GPIO9;
    let mut pin_c_6_to_0 = Output::new(peripherals.GPIO0, Level::High, OutputConfig::default());

    // INTERFACE_DEFINITIONS
    let i2c_peripheral = peripherals.I2C0;
    let mut i2c = setup_i2c!(i2c_peripheral, sda, scl, 10);

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    let header_style = MonoTextStyleBuilder::new()
        .font(&FONT_8X13)
        .text_color(BinaryColor::On)
        .build();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let border_style = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(1)
        .build();

    let rect = Rectangle::new(Point::new(0, 0), Size::new(127, 63)).into_styled(border_style);

    let header = Text::with_baseline(
        "Rust Embedded",
        Point::new(10, 3),
        header_style,
        Baseline::Top,
    );

    let mut seconds = 0;
    let mut counter_string: String<40> = String::new();
    let mut buffer = itoa::Buffer::new();

    loop {
        display.clear_buffer();
        rect.draw(&mut display).unwrap();
        header.draw(&mut display).unwrap();

        counter_string.clear();
        counter_string
            .push_str("Iron Coder Says Hi!\nSeconds: ")
            .unwrap();
        let seconds_str = buffer.format(seconds);
        counter_string.push_str(seconds_str).unwrap();

        Text::with_baseline(
            &counter_string,
            Point::new(10, 20),
            text_style,
            Baseline::Top,
        )
        .draw(&mut display)
        .unwrap();

        display.flush().unwrap();
        delay.delay(Duration::from_secs(1));
        seconds += 1;

        pin_c_6_to_0.toggle();
    }
}
