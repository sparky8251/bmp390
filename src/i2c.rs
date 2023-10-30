use embedded_hal::delay::DelayUs;
use embedded_hal::i2c::ErrorType;
use embedded_hal::i2c::I2c;

use super::{BMP390Common, Bmp390Config, CompensationData, Interface, OsrConfig, PowerConfig, BMP390_LSB_PRESSURE_REGISTER, BMP390_LSB_TEMPERATURE_REGISTER, BMP390_MSB_PRESSURE_REGISTER, BMP390_MSB_TEMPERATURE_REGISTER, BMP390_P_T_DATA_LEN, BMP390_XLSB_PRESSURE_REGISTER, BMP390_XLSB_TEMPERATURE_REGISTER, BMP390Measurement, BMP390_COMPENSATION_REGISTERS};

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
            reg += reg
        }
        Ok(CompensationData::from(out))
    }
    fn read_raw_pressure_data(&mut self) -> Result<u32, Self::Error> {
        let msb = self.read_register(BMP390_MSB_PRESSURE_REGISTER)?;
        let lsb = self.read_register(BMP390_LSB_PRESSURE_REGISTER)?;
        let xlsb = self.read_register(BMP390_XLSB_PRESSURE_REGISTER)?;
        let out: u32 = (msb as u32) << 8;
        let out: u32 = (out | lsb as u32) << 8;
        let out: u32 = out | xlsb as u32;
        Ok(out)
    }

    fn read_raw_temperature_data(&mut self) -> Result<u32, Self::Error> {
        let msb = self.read_register(BMP390_MSB_TEMPERATURE_REGISTER)?;
        let lsb = self.read_register(BMP390_LSB_TEMPERATURE_REGISTER)?;
        let xlsb = self.read_register(BMP390_XLSB_TEMPERATURE_REGISTER)?;
        let out: u32 = (msb as u32) << 8;
        let out: u32 = (out | lsb as u32) << 8;
        let out: u32 = out | xlsb as u32;
        Ok(out)
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

    pub fn soft_reset<D: DelayUs>(&mut self, delay: &mut D) -> Result<(), I2C::Error> {
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

    fn compensate_temperature(&self, raw_temperature: u32, compensation_data: &CompensationData) -> f32 {
        let comp_temp: f32;

        let partial_data1: f32;
        let partial_data2: f32;

        partial_data1 = (raw_temperature - u32::from(compensation_data.t1)) as f32;
        partial_data2 = partial_data1 * compensation_data.t2 as f32;
        comp_temp = partial_data2 + (partial_data1 * partial_data1) * compensation_data.t3 as f32;

        comp_temp
    }

    fn compensate_pressure(&self,
        raw_pressure: u32,
        compensated_temperature: f32,
        compensation_data: &CompensationData,
    ) -> f32 {
        let comp_press: f32;

        let mut partial_data1: f32;
        let mut partial_data2: f32;
        let mut partial_data3: f32;
        let partial_data4: f32;
        let partial_out1: f32;
        let partial_out2: f32;

        partial_data1 = compensation_data.p6 as f32 * compensated_temperature;
        partial_data2 =
            compensation_data.p7 as f32 * (compensated_temperature * compensated_temperature);
        partial_data3 = compensation_data.p8 as f32
            * (compensated_temperature * compensated_temperature * compensated_temperature);
        partial_out1 = compensation_data.p5 as f32 + partial_data1 + partial_data2 + partial_data3;

        partial_data1 = compensation_data.p2 as f32 * compensated_temperature;
        partial_data2 =
            compensation_data.p3 as f32 * (compensated_temperature * compensated_temperature);
        partial_data3 = compensation_data.p4 as f32
            * (compensated_temperature * compensated_temperature * compensated_temperature);
        partial_out2 = raw_pressure as f32
            * (compensation_data.p1 as f32 + partial_data1 + partial_data2 + partial_data3);

        partial_data1 = raw_pressure as f32 * raw_pressure as f32;
        partial_data2 =
            compensation_data.p9 as f32 + compensation_data.p10 as f32 * compensated_temperature;
        partial_data3 = partial_data1 + partial_data2;
        partial_data4 = partial_data3
            + (raw_pressure as f32 * raw_pressure as f32 * raw_pressure as f32)
            * compensation_data.p11 as f32;

        comp_press = partial_out1 + partial_out2 + partial_data4;

        comp_press
    }

    pub fn take_measurement(&mut self) -> Result<BMP390Measurement, I2C::Error> {
        let raw_temperature = self.common.read_raw_temperature_data()?;
        let raw_pressure = self.common.read_raw_pressure_data()?;
        let compensation_data = self.common.read_compensation_data()?;

        let compensated_temperature = self.compensate_temperature(raw_temperature, &compensation_data);
        let compensated_pressure = self.compensate_pressure(raw_pressure, compensated_temperature, &compensation_data);

        Ok(BMP390Measurement {
            temp: compensated_temperature,
            press: compensated_pressure,
        })
    }
}
