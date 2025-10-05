#![no_std]

#[cfg(feature = "rp2040")]
mod rp2040;
#[cfg(feature = "rp2040")]
pub use rp2040::*;

#[cfg(feature = "arduino")]
mod arduino;
#[cfg(feature = "arduino")]
pub use arduino::*;

#[cfg(feature = "stm32f4")]
mod stm32f4;
#[cfg(feature = "stm32f4")]
pub use stm32f4::*;

#[cfg(feature = "esp")]
mod esp;
#[cfg(feature = "esp")]
pub use esp::*;


// Ensure exactly one feature is enabled
#[cfg(not(any(feature = "rp2040", feature = "arduino", feature = "stm32f4", feature = "esp")))]
compile_error!("You must enable exactly one HAL feature: rp2040, arduino, stm32f4, or esp");

#[cfg(any(
    all(feature = "rp2040", feature = "arduino"),
    all(feature = "rp2040", feature = "stm32f4"),
    all(feature = "rp2040", feature = "esp"),
    all(feature = "arduino", feature = "stm32f4"),
    all(feature = "arduino", feature = "esp"),
    all(feature = "stm32f4", feature = "esp")
))]
compile_error!("Cannot enable multiple HAL features simultaneously");

