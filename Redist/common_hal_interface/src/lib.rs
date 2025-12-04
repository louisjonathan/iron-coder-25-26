#![no_std]

// ============================================================================
// RP2040 Support
// ============================================================================

// Raw rp2040-hal (no BSP) - only when rp2040 is enabled but no BSP feature
#[cfg(all(feature = "rp2040", not(any(
    feature = "adafruit-feather-rp2040",
    feature = "adafruit-kb2040",
    feature = "adafruit-qt-py-rp2040",
    feature = "sparkfun-pro-micro-rp2040",
    feature = "rp-pico"
))))]
mod rp2040;
#[cfg(all(feature = "rp2040", not(any(
    feature = "adafruit-feather-rp2040",
    feature = "adafruit-kb2040",
    feature = "adafruit-qt-py-rp2040",
    feature = "sparkfun-pro-micro-rp2040",
    feature = "rp-pico"
))))]
pub use rp2040::*;

// Adafruit Feather RP2040 BSP
// Note: macros are exported at crate root via #[macro_export], no pub use needed
#[cfg(feature = "adafruit-feather-rp2040")]
mod rp2040_adafruit_feather;

// Raspberry Pi Pico BSP
#[cfg(feature = "rp-pico")]
mod rp2040_pico;

// TODO: Add more RP2040 BSPs as needed
// #[cfg(feature = "adafruit-kb2040")]
// mod rp2040_kb2040;
// #[cfg(feature = "adafruit-kb2040")]
// pub use rp2040_kb2040::*;

// ============================================================================
// Arduino Support
// ============================================================================

#[cfg(feature = "arduino")]
mod arduino;
#[cfg(feature = "arduino")]
pub use arduino::*;

// ============================================================================
// STM32F4 Support
// ============================================================================

#[cfg(feature = "stm32f4")]
mod stm32f4;
#[cfg(feature = "stm32f4")]
pub use stm32f4::*;

// ============================================================================
// ESP Support
// ============================================================================

#[cfg(feature = "esp")]
mod esp;
#[cfg(feature = "esp")]
pub use esp::*;

// ============================================================================
// Compile-time checks
// ============================================================================

// Prevent multiple base HAL features from being enabled
#[cfg(any(
    all(feature = "rp2040", feature = "arduino"),
    all(feature = "rp2040", feature = "stm32f4"),
    all(feature = "rp2040", feature = "esp"),
    all(feature = "arduino", feature = "stm32f4"),
    all(feature = "arduino", feature = "esp"),
    all(feature = "stm32f4", feature = "esp")
))]
compile_error!("Cannot enable multiple HAL features simultaneously");

// Prevent multiple RP2040 BSP features from being enabled
#[cfg(any(
    all(feature = "adafruit-feather-rp2040", feature = "rp-pico"),
    all(feature = "adafruit-feather-rp2040", feature = "adafruit-kb2040"),
    all(feature = "adafruit-feather-rp2040", feature = "sparkfun-pro-micro-rp2040"),
    all(feature = "rp-pico", feature = "adafruit-kb2040"),
    all(feature = "rp-pico", feature = "sparkfun-pro-micro-rp2040"),
    all(feature = "adafruit-kb2040", feature = "sparkfun-pro-micro-rp2040")
))]
compile_error!("Cannot enable multiple RP2040 BSP features simultaneously");
