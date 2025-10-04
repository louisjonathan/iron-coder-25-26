use arduino_hal::{
    hal::port::{PB5, PD2, PD3, PD4},
    port::{
        mode::{Input, Output},
        Pin,
    },
    Delay, I2c, Peripherals, Spi,
};

extern crate alloc;
use alloc::vec::Vec;

pub type Error = ();

pub struct common_general {
    pub pins: Vec<common_pin>,
    pub spi: Vec<common_spi>,
    pub i2c: Vec<common_i2c>,
    
    delay: Delay,
    peripherals: Option<Peripherals>,
}

impl common_general {
    pub fn new() -> Self {
        let peripherals = Peripherals::take().unwrap();
        Self {
            pins: Vec::new(),
            spi: Vec::new(),
            i2c: Vec::new(),
            delay: Delay::new(),
            peripherals: Some(peripherals),
        }
    }
    
    pub fn initialize_pins(&mut self) -> Result<(), Error> {

    }
    
    pub fn initialize_spi(&mut self) -> Result<(), Error> {

    }
    
    pub fn initialize_i2c(&mut self) -> Result<(), Error> {
        
    }
    
    pub fn sleep_ms(&mut self, ms: u32) {
        self.delay.delay_ms(ms as u16);
    }
}

pub struct common_pin<M, P> {
    inner: Pin<M, P>,
}

impl common_pin {
    pub fn set_high(&mut self) {
        self.inner.set_high();
    }
    
    pub fn set_low(&mut self) {
        self.inner.set_low();
    }
    
    pub fn is_high(&self) -> bool {
        self.inner.is_high();
    }
    
    pub fn is_low(&self) -> bool {
        self.inner.is_low();
    }
}

pub struct common_spi {
    inner: Spi
}

impl common_spi {
     pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.inner.write(data).map_err(|_| ())
     }
     pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.inner.read(buffer).map_err(|_| ())
     }
     pub fn write_read(&mut self, data: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        self.inner.write_read(data, buffer).map_err(|_| ())
     }
}

pub struct common_i2c {
    inner: I2c,
}

impl common_i2c {
   pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        self.inner.write(data).map_err(|_| ())
     }
     pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), Error> {
        self.inner.read(buffer).map_err(|_| ())
     }
     pub fn write_read(&mut self, data: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        self.inner.write_read(data, buffer).map_err(|_| ())
     }

}

