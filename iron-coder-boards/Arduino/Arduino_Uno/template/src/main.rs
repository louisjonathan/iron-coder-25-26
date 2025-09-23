#![no_std]
#![no_main]

use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use panic_halt as _;
use ufmt::uwriteln;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    let mut led = pins.d11.into_output();
    let mut internal_led = pins.d13.into_output();
    let button = pins.d12.into_pull_up_input();

    loop {
        if button.is_low() {
            led.set_low();
            uwriteln!(&mut serial, "button pressed!").unwrap_infallible();
        } else {
            led.set_high();
        }
        arduino_hal::delay_ms(100);
        internal_led.toggle();
    }
}
