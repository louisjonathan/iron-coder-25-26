//! Blinks the LED on a Adafruit Feather RP2040 board
//!
//! This will blink on-board LED.
#![no_std]
#![no_main]
use common_hal_interface::*;
use adafruit_feather_rp2040::entry;
use adafruit_feather_rp2040::{
    hal::{
		self,
        clocks::{init_clocks_and_plls, Clock},
        pac,
        watchdog::Watchdog,
        Sio,
    },
    Pins, XOSC_CRYSTAL_FREQ,
};
use embedded_hal::digital::v2::OutputPin;
use panic_halt as _;

#[entry]
fn main() -> ! {
    rp2040_setup!(pac, core, clocks, sio, pins);
	let mut delay = new_delay!(core,clocks); 
    
    loop {
        delay.delay_ms(100);
    }
}