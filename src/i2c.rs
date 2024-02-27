use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::ErrorType;
use embedded_hal::i2c::I2c;

use crate::BMP390ID;

use super::{
    BMP390Common, BMP390Measurement, Bmp390Config, CompensationData, Interface, OsrConfig,
    PowerConfig, BMP390_COMPENSATION_REGISTERS, BMP390_LSB_PRESSURE_REGISTER,
    BMP390_LSB_TEMPERATURE_REGISTER, BMP390_MSB_PRESSURE_REGISTER, BMP390_MSB_TEMPERATURE_REGISTER,
    BMP390_P_T_DATA_LEN, BMP390_XLSB_PRESSURE_REGISTER, BMP390_XLSB_TEMPERATURE_REGISTER,
};

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

    fn read_compensation_data(&mut self) -> Result<CompensationData, Self::Error> {
        let mut out = [0u8; BMP390_COMPENSATION_REGISTERS];
        let mut reg: u8 = 0x31;
        for i in 0..BMP390_COMPENSATION_REGISTERS {
            out[i] = self.read_register(reg)?;
            reg += 1
        }
        Ok(CompensationData::from(out))
    }
    fn read_raw_pressure_data(&mut self) -> Result<u32, Self::Error> {
        let msb = self.read_register(BMP390_MSB_PRESSURE_REGISTER)?;
        let lsb = self.read_register(BMP390_LSB_PRESSURE_REGISTER)?;
        let xlsb = self.read_register(BMP390_XLSB_PRESSURE_REGISTER)?;
        let uncompensated_pressure = u32::from_be_bytes([0,msb,lsb,xlsb]);
        Ok(uncompensated_pressure)
    }

    fn read_raw_temperature_data(&mut self) -> Result<u32, Self::Error> {
        let msb = self.read_register(BMP390_MSB_TEMPERATURE_REGISTER)?;
        let lsb = self.read_register(BMP390_LSB_TEMPERATURE_REGISTER)?;
        let xlsb = self.read_register(BMP390_XLSB_TEMPERATURE_REGISTER)?;
        let uncompensated_temperature = u32::from_be_bytes([0,msb,lsb,xlsb]);
        Ok(uncompensated_temperature)
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

    pub fn init<D: DelayNs>(
        &mut self,
        delay: &mut D,
        config: Option<Bmp390Config>,
    ) -> Result<BMP390ID, I2C::Error> {
        self.common.init(delay, config)
    }

    pub fn soft_reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I2C::Error> {
        self.common.soft_reset(delay)
    }

    pub fn set_all_configs(&mut self, config: &Bmp390Config) -> Result<(), I2C::Error> {
        self.common.set_all_configs(config)
    }

    pub fn set_osr_config(&mut self, osr_config: &OsrConfig) -> Result<(), I2C::Error> {
        self.common.set_osr_config(osr_config)
    }

    pub fn set_power_config(&mut self, power_config: &PowerConfig) -> Result<(), I2C::Error> {
        self.common.set_power_config(power_config)
    }

    pub fn take_measurement(&mut self) -> Result<BMP390Measurement, I2C::Error> {
        self.common.take_measurement()
    }
}
