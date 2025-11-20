#![allow(warnings)]
#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use panic_halt as _;
use ufmt::uwriteln;

use common_hal_interface::*;

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

    // INTERFACE_DEFINITIONS

    loop {
        arduino_hal::delay_ms(100);
    }
}
