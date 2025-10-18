# Iron Coder's common_hal_interface


### This is a common interface between major Rust HALs such as arduino-hal, rp-hal, and esp-hal. It functions primarily as a macro-based compatibility layer between boards to make programming in Rust across boards slightly more uniform.

### An example macro is as follows:

```Rust
#[macro_export]
macro_rules! arduino_setup {
    ($baud_rate:expr, $dp:ident, $pins:ident, $serial:ident) => {
        let $dp = arduino_hal::Peripherals::take().unwrap();
        let $pins = arduino_hal::pins!($dp);
        let mut $serial = arduino_hal::default_serial!($dp, $pins, $baud_rate);
    };
}
```

#### This macro takes in four parameters and uses them to store important objects such as `dp` and `pins` for arduino, also setting up the serial communication and returning that to `serial`

### An example of its use is as follows:


```Rust
#![allow(warnings)]
#![no_std]
#![no_main]

use arduino_hal::prelude::_unwrap_infallible_UnwrapInfallible;
use panic_halt as _;

use common_hal_interface::*;

#[arduino_hal::entry]
fn main() -> ! {
    arduino_setup!(57600, dp, pins, serial);
    let mut internal_led = pins.d13.into_output();
    loop {
        arduino_hal::delay_ms(100);
        internal_led.toggle();
        arduino_hal::delay_ms(500);
        internal_led.toggle();
    }
}
```

#### As you can see, this simple interface allows novice Rust users to program in embedded rust more easily by abstracting away language and package specific differences. The pins, dp, and serial parameters to arduino_setup can be used as if the user set them up in the manner intended by the arduino-hal package creator (more verbose, confusing, package specific) without the confusion present in the intended method.

