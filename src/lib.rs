#![no_std]

use defmt::{debug, info};
use embedded_hal::delay::DelayUs;

pub mod i2c;

pub const BMP390_SLEEP_MODE: u8 = 0x0;
pub const BMP390_FORCED_MODE: u8 = 0x1;
pub const BMP390_NORMAL_MODE: u8 = 0x3;

pub const BMP390_PRESSURE_OVERSAMPLING_X1: u8 = 0x1;
pub const BMP390_PRESSURE_OVERSAMPLING_X2: u8 = 0x2;
pub const BMP390_PRESSURE_OVERSAMPLING_X4: u8 = 0x3;
pub const BMP390_PRESSURE_OVERSAMPLING_X8: u8 = 0x4;
pub const BMP390_PRESSURE_OVERSAMPLING_X16: u8 = 0x5;
pub const BMP390_PRESSURE_OVERSAMPLING_X32: u8 = 0x6;

pub const BMP390_TEMPERATURE_OVERSAMPLING_X1: u8 = 0x1;
pub const BMP390_TEMPERATURE_OVERSAMPLING_X2: u8 = 0x2;
pub const BMP390_TEMPERATURE_OVERSAMPLING_X4: u8 = 0x3;
pub const BMP390_TEMPERATURE_OVERSAMPLING_X8: u8 = 0x4;
pub const BMP390_TEMPERATURE_OVERSAMPLING_X16: u8 = 0x5;
pub const BMP390_TEMPERATURE_OVERSAMPLING_X32: u8 = 0x6;

pub const BMP390_CHIP_ID_REGISTER: u8 = 0x0;
pub const BMP390_REV_ID_REGISTER: u8 = 0x1;
pub const BMP390_ERR_REGISTER: u8 = 0x2;
pub const BMP390_STATUS_REGISTER: u8 = 0x3;

pub const BMP390_MSB_PRESSURE_REGISTER: u8 = 0x4;
pub const BMP390_LSB_PRESSURE_REGISTER: u8 = 0x5;
pub const BMP390_XLSB_PRESSURE_REGISTER: u8 = 0x6;

pub const BMP390_MSB_TEMPERATURE_REGISTER: u8 = 0x7;
pub const BMP390_LSB_TEMPERATURE_REGISTER: u8 = 0x8;
pub const BMP390_XLSB_TEMPERATURE_REGISTER: u8 = 0x9;

pub const BMP390_SENSOR_TIME_2_REGISTER: u8 = 0xC;
pub const BMP390_SENSOR_TIME_1_REGISTER: u8 = 0xD;
pub const BMP390_SENSOR_TIME_0_REGISTER: u8 = 0xE;

pub const BMP390_EVENT_REGISTER: u8 = 0x10;
pub const BMP390_INT_STATUS_REGISTER: u8 = 0x11;

pub const BMP390_FIFO_LENGTH_MSB_REGISTER: u8 = 0x12;
pub const BMP390_FIFO_LENGTH_LSB_REGISTER: u8 = 0x13;
pub const BMP390_FIFO_DATA_REGISTER: u8 = 0x14;
pub const BMP390_FIFO_WATERMARK_MSB_REGISTER: u8 = 0x15;
pub const BMP390_FIFO_WATERMARK_LSB_REGISTER: u8 = 0x16;
pub const BMP390_FIFO_CONFIG_1_REGISTER: u8 = 0x17;
pub const BMP390_FIFO_CONFIG_2_REGISTER: u8 = 0x18;

pub const BMP390_INT_CTRL_REGISTER: u8 = 0x19;
pub const BMP390_IF_CONF_REGISTER: u8 = 0x1A;
pub const BMP390_PWR_CTRL_REGISTER: u8 = 0x1B;
pub const BMP390_OSR_REGISTER: u8 = 0x1C;
pub const BMP390_ODR_REGISTER: u8 = 0x1D;
pub const BMP390_CONFIG_REGISTER: u8 = 0x1F;

// TODO: calibration data registers from 0x30 to 0x57

pub const BMP390_CMD_REGISTER: u8 = 0x7E;

const BME280_P_T_H_DATA_LEN: usize = 8;

trait Interface {
    type Error;

    fn read_register(&mut self, register: u8) -> Result<u8, Self::Error>;

    fn read_data(&mut self, register: u8) -> Result<[u8; BME280_P_T_H_DATA_LEN], Self::Error>;

    fn write_register(&mut self, register: u8, payload: u8) -> Result<(), Self::Error>;
}

struct BMP390Common<I> {
    interface: I,
    //calibration: Option<CalibrationData>
}

impl<I> BMP390Common<I>
where
    I: Interface,
{
    fn init<D: DelayUs>(&mut self, _delay: &mut D) -> Result<(), I::Error> {
        if let Err(e) = self.verify_chip_id() {
            info!("Failed in init function");
            return Err(e);
        };
        Ok(())
    }

    fn verify_chip_id(&mut self) -> Result<(), I::Error> {
        let chip_id = match self.interface.read_register(BMP390_CHIP_ID_REGISTER) {
            Ok(v) => v,
            Err(e) => {
                info!("Failed in verify chip id function");
                return Err(e);
            }
        };
        debug!("Chip ID is {:x}", chip_id);
        Ok(())
    }
}