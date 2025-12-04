//! Macros for Raspberry Pi Pico BSP
//!
//! Generic macro names - same interface as other RP2040 boards.

/// Set up the RP2040 peripherals using Raspberry Pi Pico BSP.
#[macro_export]
macro_rules! rp2040_setup {
    ($pac:ident, $core:ident, $clocks:ident, $pins:ident) => {
        let mut $pac = rp_pico::hal::pac::Peripherals::take().unwrap();
        let $core = rp_pico::hal::pac::CorePeripherals::take().unwrap();

        let mut watchdog = rp_pico::hal::Watchdog::new($pac.WATCHDOG);

        let $clocks = rp_pico::hal::clocks::init_clocks_and_plls(
            rp_pico::XOSC_CRYSTAL_FREQ,
            $pac.XOSC,
            $pac.CLOCKS,
            $pac.PLL_SYS,
            $pac.PLL_USB,
            &mut $pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = rp_pico::hal::Sio::new($pac.SIO);

        let $pins = rp_pico::Pins::new(
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
            rp_pico::hal::Clock::freq(&$clocks.system_clock).to_Hz(),
        )
    };
}

/// Create a new timer instance.
#[macro_export]
macro_rules! new_timer {
    ($pac:expr, $clocks:expr) => {
        rp_pico::hal::Timer::new($pac.TIMER, &mut $pac.RESETS, &$clocks)
    };
}

/// Set up I2C.
#[macro_export]
macro_rules! setup_i2c {
    ($pac:expr, $clocks:expr, $baudrate:expr, $i2c_block:ident, $sda:expr, $scl:expr) => {
        rp_pico::hal::I2C::i2c1(
            $pac.$i2c_block,
            $sda.into_function::<rp_pico::hal::gpio::FunctionI2C>(),
            $scl.into_function::<rp_pico::hal::gpio::FunctionI2C>(),
            rp_pico::hal::fugit::RateExtU32::kHz($baudrate / 1000),
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
        let (mut pio, $sm, _, _, _) = rp_pico::hal::pio::PIOExt::split(
            $pac.$pio_block,
            &mut $pac.RESETS,
        );
        Ws2812::new(
            $pin.into_function(),
            &mut pio,
            $sm,
            rp_pico::hal::Clock::freq(&$clocks.peripheral_clock),
            $timer.count_down(),
        )
    }};

    // Simple form using PIO0 and sm0
    ($pac:expr, $pin:expr, $clocks:expr, $timer:expr) => {{
        let (mut pio, sm0, _, _, _) = rp_pico::hal::pio::PIOExt::split(
            $pac.PIO0,
            &mut $pac.RESETS,
        );
        Ws2812::new(
            $pin.into_function(),
            &mut pio,
            sm0,
            rp_pico::hal::Clock::freq(&$clocks.peripheral_clock),
            $timer.count_down(),
        )
    }};
}

/// Set up SPI.
#[macro_export]
macro_rules! setup_spi {
    ($pac:expr, $clocks:expr, $baudrate:expr, $spi_block:ident, $mosi:expr, $miso:expr, $sck:expr, $spi_mode:expr) => {
        rp_pico::hal::Spi::<_, _, _, 8>::new(
            $pac.$spi_block,
            (
                $mosi.into_function(),
                $miso.into_function(),
                $sck.into_function(),
            ),
        )
        .init(
            &mut $pac.RESETS,
            rp_pico::hal::Clock::freq(&$clocks.peripheral_clock),
            rp_pico::hal::fugit::RateExtU32::Hz($baudrate),
            $spi_mode,
        )
    };
}

/// Set up UART.
#[macro_export]
macro_rules! setup_uart {
    ($pac:expr, $clocks:expr, $baudrate:expr, $uart_block:ident, $tx:expr, $rx:expr) => {
        rp_pico::hal::uart::UartPeripheral::new(
            $pac.$uart_block,
            ($tx.into_function(), $rx.into_function()),
            &mut $pac.RESETS,
        )
        .enable(
            rp_pico::hal::uart::UartConfig::new(
                rp_pico::hal::fugit::RateExtU32::Hz($baudrate).into(),
                rp_pico::hal::uart::DataBits::Eight,
                None,
                rp_pico::hal::uart::StopBits::One,
            ),
            rp_pico::hal::Clock::freq(&$clocks.peripheral_clock),
        )
        .unwrap()
    };
}
