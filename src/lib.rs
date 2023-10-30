#![no_std]

use defmt::println;
use embedded_hal::delay::DelayUs;

pub mod i2c;

pub struct BMP390Measurement {
    pub temp: f32,
    pub press: f32
}

#[derive(Default, Copy, Clone)]
pub enum PowerMode {
    #[default]
    Sleep,
    Forced,
    Normal,
}

impl From<PowerMode> for u8 {
    fn from(value: PowerMode) -> Self {
        match value {
            PowerMode::Sleep => 0b00,
            PowerMode::Forced => 0b01,
            PowerMode::Normal => 0b11,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct PowerConfig {
    pub pressure_enable: bool,
    pub temperature_enable: bool,
    pub power_mode: PowerMode,
}

impl PowerConfig {
    pub fn to_u8(&self) -> u8 {
        (((self.power_mode as u8) << 4) | self.temperature_enable as u8) << 1
            | self.pressure_enable as u8
    }
    pub fn from_u8(_power_config: u8) -> Self {
        todo!()
    }
}

#[derive(Copy, Clone)]
pub enum Oversampling {
    X1,
    X2,
    X4,
    X8,
    X16,
    X32,
}

impl From<Oversampling> for u8 {
    fn from(value: Oversampling) -> Self {
        match value {
            Oversampling::X1 => 0b000,
            Oversampling::X2 => 0b001,
            Oversampling::X4 => 0b010,
            Oversampling::X8 => 0b011,
            Oversampling::X16 => 0b100,
            Oversampling::X32 => 0b101,
        }
    }
}

pub struct OsrConfig {
    pub pressure: Oversampling,
    pub temperature: Oversampling,
}

impl Default for OsrConfig {
    fn default() -> Self {
        Self {
            pressure: Oversampling::X4,
            temperature: Oversampling::X1,
        }
    }
}

impl OsrConfig {
    pub fn to_u8(&self) -> u8 {
        ((self.temperature as u8) << 3) | self.pressure as u8
    }

    pub fn from_u8(_osr_config: u8) -> Self {
        todo!()
    }
}

#[derive(Default)]
pub struct Bmp390Config {
    pub osr_config: OsrConfig,
    pub power_config: PowerConfig,
}

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

const BMP390_COMPENSATION_REGISTERS: usize = 21;

struct CompensationData {
    t1: u16,
    t2: u16,
    t3: i8,
    p1: i16,
    p2: i16,
    p3: i8,
    p4: i8,
    p5: u16,
    p6: u16,
    p7: i8,
    p8: i8,
    p9: i16,
    p10: i8,
    p11: i8,
}

impl From<[u8; BMP390_COMPENSATION_REGISTERS]> for CompensationData {
    fn from(value: [u8; BMP390_COMPENSATION_REGISTERS]) -> Self {
        CompensationData {
            t1: ((value[0] as u16) << 8) | value[1] as u16,
            t2: ((value[2] as u16) << 8) | value[3] as u16,
            t3: value[4] as i8,
            p1: ((value[5] as i16) << 8) | value[6] as i16,
            p2: ((value[7] as i16) << 8) | value[8] as i16,
            p3: value[9] as i8,
            p4: value[10] as i8,
            p5: ((value[11] as u16) << 8) | value[12] as u16,
            p6: ((value[13] as u16) << 8) | value[14] as u16,
            p7: value[15] as i8,
            p8: value[16] as i8,
            p9: ((value[17] as i16) << 8) | value[18] as i16,
            p10: value[19] as i8,
            p11: value[20] as i8,
        }
    }
}

pub const BMP390_CMD_REGISTER: u8 = 0x7E;

const BMP390_P_T_DATA_LEN: usize = 8;

trait Interface {
    type Error;

    fn read_register(&mut self, register: u8) -> Result<u8, Self::Error>;

    fn read_data(&mut self, register: u8) -> Result<[u8; BMP390_P_T_DATA_LEN], Self::Error>;

    fn write_register(&mut self, register: u8, payload: u8) -> Result<(), Self::Error>;

    fn read_compensation_data(&mut self) -> Result<CompensationData, Self::Error>;

    fn read_raw_pressure_data(&mut self) -> Result<u32, Self::Error>;

    fn read_raw_temperature_data(&mut self) -> Result<u32, Self::Error>;
}

struct BMP390Common<I> {
    interface: I,
    //calibration: Option<CalibrationData>
}

impl<I> BMP390Common<I>
where
    I: Interface,
{
    fn init<D: DelayUs>(
        &mut self,
        delay: &mut D,
        config: Option<Bmp390Config>,
    ) -> Result<(), I::Error> {
        self.soft_reset(delay)?;
        let chip_id = self.read_chip_id()?;
        let rev_id = self.read_revision_id()?;
        println!("Chip ID and Rev is ID: {}, Rev: {}", chip_id, rev_id);
        match config {
            Some(v) => self.set_all_configs(&v)?,
            None => self.set_all_configs(&Bmp390Config::default())?,
        }
        Ok(())
    }

    fn read_chip_id(&mut self) -> Result<u8, I::Error> {
        self.interface.read_register(BMP390_CHIP_ID_REGISTER)
    }

    fn read_revision_id(&mut self) -> Result<u8, I::Error> {
        self.interface.read_register(BMP390_REV_ID_REGISTER)
    }

    fn soft_reset<D: DelayUs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
        self.interface.write_register(BMP390_CMD_REGISTER, 0xB6)?;
        delay.delay_ms(4); // Double the documented reboot time, just to be extra sure its done
        Ok(())
    }

    fn set_all_configs(&mut self, config: &Bmp390Config) -> Result<(), I::Error> {
        self.set_osr_config(&config.osr_config)?;
        self.set_power_config(&config.power_config)?;
        Ok(())
    }

    fn set_osr_config(&mut self, osr_config: &OsrConfig) -> Result<(), I::Error> {
        self.interface
            .write_register(BMP390_OSR_REGISTER, osr_config.to_u8())
    }

    fn set_power_config(&mut self, power_config: &PowerConfig) -> Result<(), I::Error> {
        self.interface
            .write_register(BMP390_PWR_CTRL_REGISTER, power_config.to_u8())
    }

    fn read_raw_pressure_data(&mut self) -> Result<u32, I::Error> {
        self.interface.read_raw_pressure_data()
    }

    fn read_raw_temperature_data(&mut self) -> Result<u32, I::Error> {
        self.interface.read_raw_temperature_data()
    }

    fn read_compensation_data(&mut self) -> Result<CompensationData, I::Error> {
        self.interface.read_compensation_data()
    }
}
