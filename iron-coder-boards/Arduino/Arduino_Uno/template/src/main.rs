#![allow(warnings)]
#![no_std]
#![no_main]

use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use panic_halt as _;
use ufmt::uwriteln;

use common_hal_interface::*;

#[arduino_hal::entry]
fn main() -> ! {
    arduino_setup!(57600, dp, pins, serial);
    uwriteln!(serial, "Starting up...").unwrap();
    let i2c = setup_i2c_instance!(dp, pins, 100_000);
    let mut spi = setup_spi_instance!(dp, pins);
    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    //lol apparently the internal LED is the same pin as SPI SCK so you cant use them at the same time
    //let mut internal_led = pins.d13.into_output();
    let mut external_led = pins.d9.into_output();

    loop {
        arduino_hal::delay_ms(100);
        external_led.toggle();
        arduino_hal::delay_ms(500);
        external_led.toggle();
    }
}
