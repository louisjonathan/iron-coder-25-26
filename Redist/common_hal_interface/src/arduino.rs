//! Macros for Arduino boards using arduino-hal

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

#[macro_export]
macro_rules! setup_i2c {
    ($dp:expr, $sda:expr, $scl:expr, $freq:expr) => {{
        arduino_hal::I2c::new(
            $dp.TWI,
            $sda.into_pull_up_input(),
            $scl.into_pull_up_input(),
            $freq,
        )
    }};
}

#[macro_export]
macro_rules! setup_spi {
    ($dp:expr, $sck:expr, $mosi:expr, $miso:expr, $ss:expr) => {{
        arduino_hal::Spi::new(
            $dp.SPI,
            $sck.into_output(),
            $mosi.into_output(),
            $miso.into_pull_up_input(),
            $ss.into_output(),
            arduino_hal::spi::Settings::default(),
        )
    }};
}
