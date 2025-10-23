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
#[macro_export]
macro_rules! new_delay {
    ($core:expr, $clocks:expr) => {
		cortex_m::delay::Delay::new(
            $core.SYST,
            $clocks.system_clock.freq().to_Hz()
        )
    };
}
#[macro_export]
macro_rules! new_timer {
	($pac:expr,$clocks:expr)=>{
		hal::timer::Timer::new($pac.TIMER, &mut $pac.RESETS, &$clocks)
	}
}
#[cfg(feature = "adafruit-feather-rp2040")]
#[macro_export]
macro_rules! setup_onboard_neopixel {
    ($pac:expr, $pins:expr, $clocks:expr, $timer:expr) => {
        setup_neopixel!($pac, $pins.gpio16, $clocks, $timer)
    };
}
#[cfg(feature = "micromod-rp2040")]
macro_rules! setup_onboard_neopixel {
    ($pac:expr, $pins:expr, $clocks:expr, $timer:expr) => {
        setup_neopixel!($pac, $pins.gpio10, $clocks, $timer)
    };
}
#[macro_export]
macro_rules! setup_neopixel {
    // specify everything
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
    
    // use defaults 
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
#[macro_export]
macro_rules! setup_spi {
    ($pac:expr, $miso:expr, $mosi:expr, $sck:expr, $clocks:expr, $baudrate:expr, $spi_mode:expr) => {
        hal::spi::Spi::<_, _, _, 8>::new(
            $pac.SPI0,  // or allow user to specify SPI0/SPI1?
            ($mosi.into_function(), $miso.into_function(), $sck.into_function())
        ).init(
            &mut $pac.RESETS,
            $clocks.peripheral_clock.freq(),
            $baudrate,
            $spi_mode,
        )
    };
}

#[macro_export]
macro_rules! setup_i2c {
    ($pac:expr, $clocks:expr, $baudrate:expr, $i2c_module:expr, $sda:expr, $scl:expr) => {
        hal::i2c::I2C::new_controller(
            $pac.$i2c_module,
            $sda.into_function(),
            $scl.into_function(),
            $baudrate.Hz(),
            &mut $pac.RESETS,
            $clocks.system_clock.freq(),
        )
    };
}