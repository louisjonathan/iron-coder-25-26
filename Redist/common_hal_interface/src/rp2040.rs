//! Macros for raw rp2040-hal (no BSP)
//!
//! Use these macros when working directly with rp2040-hal without a board support package.
//! For BSP-specific macros, enable the appropriate feature (e.g., `adafruit-feather-rp2040`).

/// Set up the RP2040 peripherals using raw rp2040-hal.
///
/// Expects `hal` and `pac` to be in scope (typically via `use rp2040_hal as hal`).
#[macro_export]
macro_rules! rp2040_setup {
    ($pac:ident, $core:ident, $clocks:ident, $sio:ident, $pins:ident) => {
        let mut $pac = pac::Peripherals::take().unwrap();
        let $core = pac::CorePeripherals::take().unwrap();

        let mut watchdog = hal::Watchdog::new($pac.WATCHDOG);

        let $clocks = hal::clocks::init_clocks_and_plls(
            XOSC_CRYSTAL_FREQ,
            $pac.XOSC,
            $pac.CLOCKS,
            $pac.PLL_SYS,
            $pac.PLL_USB,
            &mut $pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let $sio = hal::Sio::new($pac.SIO);

        let $pins = hal::gpio::Pins::new(
            $pac.IO_BANK0,
            $pac.PADS_BANK0,
            $sio.gpio_bank0,
            &mut $pac.RESETS,
        );
    };
}

/// Create a new delay instance.
#[macro_export]
macro_rules! new_delay {
    ($core:expr, $clocks:expr) => {
        cortex_m::delay::Delay::new($core.SYST, $clocks.system_clock.freq().to_Hz())
    };
}

/// Create a new timer instance.
#[macro_export]
macro_rules! new_timer {
    ($pac:expr, $clocks:expr) => {
        hal::timer::Timer::new($pac.TIMER, &mut $pac.RESETS, &$clocks)
    };
}

/// Set up a WS2812 NeoPixel.
#[macro_export]
macro_rules! setup_neopixel {
    // Full form with PIO block specification
    ($pac:expr, $pin:expr, $pio_block:ident, $sm:ident, $clocks:expr, $timer:expr) => {
        let (mut pio, $sm, _, _, _) = $pac.$pio_block.split(&mut $pac.RESETS);
        Ws2812::new(
            $pin.into_function(),
            &mut pio,
            $sm,
            $clocks.peripheral_clock.freq(),
            $timer.count_down(),
        )
    };

    // Simple form using PIO0 and sm0
    ($pac:expr, $pin:expr, $clocks:expr, $timer:expr) => {
        let (mut pio, sm0, _, _, _) = $pac.PIO0.split(&mut $pac.RESETS);
        Ws2812::new(
            $pin.into_function(),
            &mut pio,
            sm0,
            $clocks.peripheral_clock.freq(),
            $timer.count_down(),
        )
    };
}

/// Set up SPI.
#[macro_export]
macro_rules! setup_spi {
    ($pac:expr, $miso:expr, $mosi:expr, $sck:expr, $clocks:expr, $baudrate:expr, $spi_module:expr, $spi_mode:expr) => {
        hal::spi::Spi::<_, _, _, 8>::new(
            $spi_module,
            (
                $mosi.into_function(),
                $miso.into_function(),
                $sck.into_function(),
            ),
        )
        .init(
            &mut $pac.RESETS,
            $clocks.peripheral_clock.freq(),
            $baudrate.Hz(),
            $spi_mode,
        )
    };
}

/// Set up I2C.
#[macro_export]
macro_rules! setup_i2c {
    ($pac:expr, $clocks:expr, $baudrate:expr, $i2c_module:ident, $sda:expr, $scl:expr) => {
        hal::i2c::I2C::new_controller(
            $i2c_module,
            $sda.into_function(),
            $scl.into_function(),
            $baudrate.Hz(),
            &mut $pac.RESETS,
            $clocks.system_clock.freq(),
        )
    };
}
