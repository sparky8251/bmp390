use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::ErrorType;
use embedded_hal::i2c::I2c;

use super::{BMP390Common, Bmp390Config, Interface, BMP390_P_T_DATA_LEN};

#[derive(Copy, Clone)]
enum Bmp390Address {
    Primary,
    Secondary,
}

impl From<Bmp390Address> for u8 {
    fn from(value: Bmp390Address) -> Self {
        match value {
            Bmp390Address::Primary => 0x77,
            Bmp390Address::Secondary => 0x76,
        }
    }
}

struct I2CInterface<I2C> {
    i2c: I2C,
    address: Bmp390Address,
}

impl<I2C> Interface for I2CInterface<I2C>
where
    I2C: I2c + ErrorType,
{
    type Error = I2C::Error;

    fn read_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        let mut data: [u8; 1] = [0];
        self.i2c
            .write_read(self.address.into(), &[register], &mut data)?;
        Ok(data[0])
    }

    fn read_data(&mut self, register: u8) -> Result<[u8; BMP390_P_T_DATA_LEN], Self::Error> {
        let mut data = [0; BMP390_P_T_DATA_LEN];
        self.i2c
            .write_read(self.address.into(), &[register], &mut data)?;
        Ok(data)
    }

    fn write_register(&mut self, register: u8, payload: u8) -> Result<(), Self::Error> {
        self.i2c.write(self.address.into(), &[register, payload])
    }
}

pub struct BMP390<I2C> {
    common: BMP390Common<I2CInterface<I2C>>,
}

impl<I2C> BMP390<I2C>
where
    I2C: I2c + ErrorType,
{
    fn new(i2c: I2C, address: Bmp390Address) -> Self {
        Self {
            common: BMP390Common {
                interface: I2CInterface { i2c, address },
            },
        }
    }

    pub fn new_primary(i2c: I2C) -> Self {
        Self::new(i2c, Bmp390Address::Primary)
    }

    pub fn new_secondary(i2c: I2C) -> Self {
        Self::new(i2c, Bmp390Address::Secondary)
    }

    pub fn init<D: DelayUs>(
        &mut self,
        delay: &mut D,
        config: Option<Bmp390Config>,
    ) -> Result<(), I2C::Error> {
        self.common.init(delay, config)
    }
}
