#[macro_export]
macro_rules! arduino_setup {
    ($dp:ident, $pins:ident) => {
        let $dp = arduino_hal::Peripherals::take().unwrap();
        let $pins = arduino_hal::pins!($dp);
    };
}
#[macro_export]
macro_rules! setup_serial {
    ($dp:expr, $pins:expr, $baud_rate:expr) => {
        arduino_hal::default_serial!($dp, $pins, $baud_rate)
    };
}

// #[cfg(feature = "arduino-uno")]
// #[macro_export]
// macro_rules! setup_spi {
//     ($pins:expr) => {{
//         (
//             $pins.d13.into_output(),        // SCK
//             $pins.d11.into_output(),        // MOSI
//             $pins.d12.into_pull_up_input(), // MISO
//             $pins.d10.into_output(),        // SS
//         )
//     }};
// }
// #[cfg(feature = "arduino-mega")]
// #[macro_export]
// macro_rules! setup_spi {
//     ($pins:expr) => {{
//         {
//             (
//                 $pins.d52.into_output(),        // SCK
//                 $pins.d51.into_output(),        // MOSI
//                 $pins.d50.into_pull_up_input(), // MISO
//                 $pins.d53.into_output(),        // SS
//             )
//         }
//     }};
// }
// #[cfg(feature = "arduino-nano")]
// #[macro_export]
// macro_rules! setup_spi {
//     ($pins:expr) => {{
//         {
//             (
//                 $pins.d13.into_output(),        // SCK
//                 $pins.d11.into_output(),        // MOSI
//                 $pins.d12.into_pull_up_input(), // MISO
//                 $pins.d10.into_output(),        // SS
//             )
//         }
//     }};
// }
// #[cfg(any(
//     feature = "arduino-leonardo",
//     feature = "arduino-micro",
//     feature = "pro-micro"
// ))]
// #[macro_export]
// macro_rules! setup_spi {
//     ($pins:expr) => {{
//         {
//             (
//                 $pins.d3.into_output(),        // SCK
//                 $pins.d2.into_output(),        // MOSI
//                 $pins.d0.into_pull_up_input(), // MISO
//                 $pins.d1.into_output(),        // SS
//             )
//         }
//     }};
// }

// #[cfg(feature = "nano-33-ble")]
// #[macro_export]
// macro_rules! setup_spi {
//     ($pins:expr) => {{
//         (
//             $pins.sck.into_output(),         // SCK
//             $pins.mosi.into_output(),        // MOSI
//             $pins.miso.into_pull_up_input(), // MISO
//             $pins.ss.into_output(),          // SS
//         )
//     }};
// }

// #[cfg(any(feature = "arduino-uno", feature = "arduino-nano"))]
// #[macro_export]
// macro_rules! setup_i2c {
//     ($pins:expr) => {
//         (
//             $pins.a4.into_floating_input().into_pull_up_input(), // SDA
//             $pins.a5.into_floating_input().into_pull_up_input(), // SCL
//         )
//     };
// }

// #[cfg(feature = "arduino-mega")]
// #[macro_export]
// macro_rules! setup_i2c {
//     ($pins:expr) => {
//         (
//             $pins.d20.into_floating_input().into_pull_up_input(), // SDA
//             $pins.d21.into_floating_input().into_pull_up_input(), // SCL
//         )
//     };
// }

// #[cfg(any(
//     feature = "arduino-leonardo",
//     feature = "arduino-micro",
//     feature = "pro-micro"
// ))]
// #[macro_export]
// macro_rules! setup_i2c {
//     ($pins:expr) => {
//         (
//             $pins.d2.into_floating_input().into_pull_up_input(), // SDA
//             $pins.d3.into_floating_input().into_pull_up_input(), // SCL
//         )
//     };
// }

// #[cfg(feature = "nano-33-ble")]
// #[macro_export]
// macro_rules! setup_i2c {
//     ($pins:expr) => {
//         (
//             $pins.sda.into_floating_input().into_pull_up_input(), // SDA
//             $pins.scl.into_floating_input().into_pull_up_input(), // SCL
//         )
//     };
// }
//
#[macro_export]
macro_rules! setup_i2c {
    ($dp:expr, $sda:expr, $scl:expr, $freq:expr) => {{ arduino_hal::I2c::new($dp.TWI, $sda, $scl, $freq) }};
}

#[macro_export]
macro_rules! setup_spi {
    ($dp:expr, $sck:expr, $mosi:expr, $ss:expr) => {{
        let (sck, mosi, miso, ss) = setup_spi!($pins);
        arduino_hal::Spi::new(
            $dp.SPI,
            $sck,
            $mosi,
            $miso,
            $ss,
            arduino_hal::spi::Settings::default(),
        )
    }};
}
