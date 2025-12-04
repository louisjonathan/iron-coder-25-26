//! Macros for Adafruit Feather RP2040 BSP
//!
//! Generic macro names - same interface as other RP2040 boards.

/// Set up the RP2040 peripherals using Adafruit Feather RP2040 BSP.
#[macro_export]
macro_rules! rp2040_setup {
    ($pac:ident, $core:ident, $clocks:ident, $pins:ident) => {
        let mut $pac = adafruit_feather_rp2040::hal::pac::Peripherals::take().unwrap();
        let $core = adafruit_feather_rp2040::hal::pac::CorePeripherals::take().unwrap();

        let mut watchdog = adafruit_feather_rp2040::hal::Watchdog::new($pac.WATCHDOG);

        let $clocks = adafruit_feather_rp2040::hal::clocks::init_clocks_and_plls(
            adafruit_feather_rp2040::XOSC_CRYSTAL_FREQ,
            $pac.XOSC,
            $pac.CLOCKS,
            $pac.PLL_SYS,
            $pac.PLL_USB,
            &mut $pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = adafruit_feather_rp2040::hal::Sio::new($pac.SIO);

        let $pins = adafruit_feather_rp2040::Pins::new(
            $pac.IO_BANK0,
            $pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut $pac.RESETS,
        );
    };
}

/// Create a new delay instance.
#[macro_export]
macro_rules! new_delay {
    ($core:expr, $clocks:expr) => {
        cortex_m::delay::Delay::new(
            $core.SYST,
            adafruit_feather_rp2040::hal::Clock::freq(&$clocks.system_clock).to_Hz(),
        )
    };
}

/// Create a new timer instance.
#[macro_export]
macro_rules! new_timer {
    ($pac:expr, $clocks:expr) => {
        adafruit_feather_rp2040::hal::Timer::new($pac.TIMER, &mut $pac.RESETS, &$clocks)
    };
}

/// Set up I2C.
#[macro_export]
macro_rules! setup_i2c {
    ($pac:expr, $clocks:expr, $baudrate:expr, $i2c_block:ident, $sda:expr, $scl:expr) => {
        adafruit_feather_rp2040::hal::I2C::i2c1(
            $pac.$i2c_block,
            $sda.into_function::<adafruit_feather_rp2040::hal::gpio::FunctionI2C>(),
            $scl.into_function::<adafruit_feather_rp2040::hal::gpio::FunctionI2C>(),
            adafruit_feather_rp2040::hal::fugit::RateExtU32::kHz($baudrate / 1000),
            &mut $pac.RESETS,
            &$clocks.system_clock,
        )
    };
}

/// Set up a WS2812 NeoPixel.
#[macro_export]
macro_rules! setup_neopixel {
    // Full form with PIO block specification
    ($pac:expr, $pin:expr, $pio_block:ident, $sm:ident, $clocks:expr, $timer:expr) => {{
        let (mut pio, $sm, _, _, _) = adafruit_feather_rp2040::hal::pio::PIOExt::split(
            $pac.$pio_block,
            &mut $pac.RESETS,
        );
        Ws2812::new(
            $pin.into_function(),
            &mut pio,
            $sm,
            adafruit_feather_rp2040::hal::Clock::freq(&$clocks.peripheral_clock),
            $timer.count_down(),
        )
    }};

    // Simple form using PIO0 and sm0
    ($pac:expr, $pin:expr, $clocks:expr, $timer:expr) => {{
        let (mut pio, sm0, _, _, _) = adafruit_feather_rp2040::hal::pio::PIOExt::split(
            $pac.PIO0,
            &mut $pac.RESETS,
        );
        Ws2812::new(
            $pin.into_function(),
            &mut pio,
            sm0,
            adafruit_feather_rp2040::hal::Clock::freq(&$clocks.peripheral_clock),
            $timer.count_down(),
        )
    }};
}

/// Set up the onboard NeoPixel (GPIO16 on Feather RP2040).
#[macro_export]
macro_rules! setup_onboard_neopixel {
    ($pac:expr, $pins:expr, $clocks:expr, $timer:expr) => {
        setup_neopixel!($pac, $pins.neopixel, $clocks, $timer)
    };
}

/// Set up SPI.
#[macro_export]
macro_rules! setup_spi {
    ($pac:expr, $clocks:expr, $baudrate:expr, $spi_block:ident, $mosi:expr, $miso:expr, $sck:expr, $spi_mode:expr) => {
        adafruit_feather_rp2040::hal::Spi::<_, _, _, 8>::new(
            $pac.$spi_block,
            (
                $mosi.into_function(),
                $miso.into_function(),
                $sck.into_function(),
            ),
        )
        .init(
            &mut $pac.RESETS,
            adafruit_feather_rp2040::hal::Clock::freq(&$clocks.peripheral_clock),
            adafruit_feather_rp2040::hal::fugit::RateExtU32::Hz($baudrate),
            $spi_mode,
        )
    };
}

/// Set up UART.
#[macro_export]
macro_rules! setup_uart {
    ($pac:expr, $clocks:expr, $baudrate:expr, $uart_block:ident, $tx:expr, $rx:expr) => {
        adafruit_feather_rp2040::hal::uart::UartPeripheral::new(
            $pac.$uart_block,
            ($tx.into_function(), $rx.into_function()),
            &mut $pac.RESETS,
        )
        .enable(
            adafruit_feather_rp2040::hal::uart::UartConfig::new(
                adafruit_feather_rp2040::hal::fugit::RateExtU32::Hz($baudrate).into(),
                adafruit_feather_rp2040::hal::uart::DataBits::Eight,
                None,
                adafruit_feather_rp2040::hal::uart::StopBits::One,
            ),
            adafruit_feather_rp2040::hal::Clock::freq(&$clocks.peripheral_clock),
        )
        .unwrap()
    };
}
