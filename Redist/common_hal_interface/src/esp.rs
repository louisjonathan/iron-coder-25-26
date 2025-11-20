#[macro_export]
macro_rules! esp_setup {
    () => {{ esp_hal::init(esp_hal::Config::default()) }};
}

#[macro_export]
macro_rules! delay_ms {
    ($ms:expr) => {
        let start = esp_hal::time::Instant::now();
        while start.elapsed() < esp_hal::time::Duration::from_millis($ms) {}
    };
}

#[macro_export]
macro_rules! esp_with_serial_setup {
    () => {{
        esp_println::logger::init_logger_from_env();
        esp_setup!()
    }};
}

#[macro_export]
macro_rules! setup_spi {
    ($spi_peripheral:expr, $sck:expr, $mosi:expr, $miso:expr, $freq_khz:expr) => {{
        let miso_clone = unsafe { $miso.clone_unchecked() };
        esp_hal::spi::master::Spi::new(
            $spi_peripheral,
            esp_hal::spi::master::Config::default()
                .with_frequency(esp_hal::time::Rate::from_khz($freq_khz))
                .with_mode(esp_hal::spi::Mode::_0),
        )
        .unwrap()
        .with_sck($sck)
        .with_mosi($mosi)
        .with_miso(miso_clone)
    }};
}

#[macro_export]
macro_rules! setup_i2c {
    ($i2c_peripheral:expr, $sda:expr, $scl:expr, $freq_khz:expr) => {{
        esp_hal::i2c::master::I2c::new(
            $i2c_peripheral,
            esp_hal::i2c::master::Config::default()
                .with_frequency(esp_hal::time::Rate::from_khz($freq_khz)),
        )
        .unwrap()
        .with_sda($sda)
        .with_scl($scl)
    }};
}
