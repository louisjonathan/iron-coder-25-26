#![allow(warnings)]
#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use ufmt::uwriteln;

use common_hal_interface::*;
use embedded_graphics::{
    mono_font::{ascii::*, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::{Baseline, Text},
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

#[arduino_hal::entry]
fn main() -> ! {
    arduino_setup!(dp, pins);
    let mut serial = setup_serial!(dp, pins, 57600);
    uwriteln!(serial, "Starting up...").unwrap();

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    // PIN_DEFINITIONS
    let scl = pins.a5;
    let sda = pins.a4;
    let mut pin_c_18_to_0 = pins.d13.into_output();

    // INTERFACE_DEFINITIONS
    let mut i2c = setup_i2c!(dp, sda, scl, 10_000);

    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    display.clear_buffer();

    let header_style = MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_4X6)
        .text_color(BinaryColor::On)
        .build();

    let border_style = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(1)
        .build();

    uwriteln!(serial, "Drawing initializer rectangle...").unwrap();
    Rectangle::new(Point::new(0, 0), Size::new(127, 63))
        .into_styled(border_style)
        .draw(&mut display)
        .unwrap();

    Text::with_baseline(
        "Rust Embedded",
        Point::new(10, 8),
        header_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    Text::with_baseline(
        "Iron Coder says Hi!",
        Point::new(10, 40),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)
    .unwrap();

    display.flush().unwrap();

    loop {
        arduino_hal::delay_ms(1000);
        pin_c_18_to_0.toggle();
        uwriteln!(serial, "running!").unwrap();
    }
}
