use defmt::info;
use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::I2c;
use embedded_hal::i2c::ErrorType;

use super::{BME280_P_T_H_DATA_LEN, BMP390Common, Interface};

const BMP390_PRIMARY_ADDRESS: u8 = 0x77;
const BMP390_SECONDARY_ADDRESS: u8 = 0x76;

struct I2CInterface<I2C> {
    i2c: I2C,
    address: u8
}

impl<I2C> Interface for I2CInterface<I2C>
where
    I2C: I2c + ErrorType,
{
    type Error = I2C::Error;

    fn read_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        let mut data: [u8; 1] = [0];
        if let Err(e) = self.i2c.write_read(self.address, &[register], &mut data) {
            info!("Failed in read register function");
            return Err(e);
        };
        Ok(data[0])
    }

    fn read_data(&mut self, register: u8) -> Result<[u8; BME280_P_T_H_DATA_LEN], Self::Error> {
        let mut data = [0; BME280_P_T_H_DATA_LEN];
        self.i2c.write_read(self.address, &[register], &mut data)?;
        Ok(data)
    }

    fn write_register(&mut self, register: u8, payload: u8) -> Result<(), Self::Error> {
        Ok(self.i2c.write(self.address, &[register, payload])?)
    }
}

pub struct BMP390<I2C> {
    common: BMP390Common<I2CInterface<I2C>>
}

impl<I2C> BMP390<I2C>
where
    I2C: I2c + ErrorType
{
    fn new(i2c: I2C, address: u8) -> Self {
        Self {
            common: BMP390Common {
                interface: I2CInterface { i2c, address }
            }
        }
    }

    pub fn new_primary(i2c: I2C) -> Self {
        Self::new(i2c, BMP390_PRIMARY_ADDRESS)
    }

    pub fn new_secondary(i2c: I2C) -> Self {
        Self::new(i2c, BMP390_SECONDARY_ADDRESS)
    }

    pub fn init<D: DelayUs>(&mut self, delay: &mut D) -> Result<(), I2C::Error> {
        self.common.init(delay)
    }
}