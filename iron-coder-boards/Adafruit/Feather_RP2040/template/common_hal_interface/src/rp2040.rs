use rp2040_hal;

#[macro_export]
macro_rules! rp2040_setup {
    () => {{
        let mut pac = pac::Peripherals::take().unwrap();
        let core = pac::CorePeripherals::take().unwrap();

        // Set up the watchdog driver - needed by the clock setup code
        let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

        // Configure the clocks
        //
        // The default is to generate a 125 MHz system clock
        let clocks = hal::clocks::init_clocks_and_plls(
            hal::XTAL_FREQ_HZ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .unwrap();

        // The single-cycle I/O block controls our GPIO pins
        let sio = hal::Sio::new(pac.SIO);

        // Set the pins up according to their function on this particular board
        let pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
        (
            pac,
            core,
            clocks,
            sio,
            pins,
        )
    }};
}

// macro_rules! rp2040_setup {
//     ($pac:ident, $core:ident, $sio:ident, $pio, $pins:ident) => {
//         let mut $pac = pac::Peripherals::take().unwrap();
//         let $core = pac::CorePeripherals::take().unwrap();

//         // Set up the watchdog driver - needed by the clock setup code
//         let mut $watchdog = hal::Watchdog::new(pac.WATCHDOG);

//         // Configure the clocks
//         //
//         // The default is to generate a 125 MHz system clock
//         let clocks = hal::clocks::init_clocks_and_plls(
//             XTAL_FREQ_HZ,
//             pac.XOSC,
//             pac.CLOCKS,
//             pac.PLL_SYS,
//             pac.PLL_USB,
//             &mut pac.RESETS,
//             &mut watchdog,
//         )
//         .unwrap();

//         // The single-cycle I/O block controls our GPIO pins
//         let sio = hal::Sio::new(pac.SIO);

//         // Set the pins up according to their function on this particular board
//         let pins = hal::gpio::Pins::new(
//             pac.IO_BANK0,
//             pac.PADS_BANK0,
//             sio.gpio_bank0,
//             &mut pac.RESETS,
//         );
//     };
// }
