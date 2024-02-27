#![no_std]

use embedded_hal::delay::DelayNs;
use libm::exp2f;

pub mod i2c;

pub struct BMP390Measurement {
    /// in celsius
    pub temp: f32,
    /// in pascals
    pub press: f32,
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

pub const BMP390_XLSB_PRESSURE_REGISTER: u8 = 0x4;
pub const BMP390_LSB_PRESSURE_REGISTER: u8 = 0x5;
pub const BMP390_MSB_PRESSURE_REGISTER: u8 = 0x6;

pub const BMP390_XLSB_TEMPERATURE_REGISTER: u8 = 0x7;
pub const BMP390_LSB_TEMPERATURE_REGISTER: u8 = 0x8;
pub const BMP390_MSB_TEMPERATURE_REGISTER: u8 = 0x9;

pub const BMP390_SENSOR_TIME_2_REGISTER: u8 = 0xC;
pub const BMP390_SENSOR_TIME_1_REGISTER: u8 = 0xD;
pub const BMP390_SENSOR_TIME_0_REGISTER: u8 = 0xE;

pub const BMP390_EVENT_REGISTER: u8 = 0x10;
pub const BMP390_INT_STATUS_REGISTER: u8 = 0x11;

pub const BMP390_FIFO_LENGTH_LSB_REGISTER: u8 = 0x12;
pub const BMP390_FIFO_LENGTH_MSB_REGISTER: u8 = 0x13;
pub const BMP390_FIFO_DATA_REGISTER: u8 = 0x14;
pub const BMP390_FIFO_WATERMARK_LSB_REGISTER: u8 = 0x15;
pub const BMP390_FIFO_WATERMARK_MSB_REGISTER: u8 = 0x16;
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
    t1: f32,
    t2: f32,
    t3: f32,
    p1: f32,
    p2: f32,
    p3: f32,
    p4: f32,
    p5: f32,
    p6: f32,
    p7: f32,
    p8: f32,
    p9: f32,
    p10: f32,
    p11: f32,
}

impl From<[u8; BMP390_COMPENSATION_REGISTERS]> for CompensationData {
    fn from(value: [u8; BMP390_COMPENSATION_REGISTERS]) -> Self {
        CompensationData {
            t1: f32::from(u16::from_le_bytes([value[0], value[1]])) / exp2f(-8.0),
            t2: f32::from(u16::from_le_bytes([value[2], value[3]])) / exp2f(30.0),
            t3: f32::from(i8::from_le_bytes([value[4]])) / exp2f(48.0),
            p1: (f32::from(i16::from_le_bytes([value[5], value[6]])) - exp2f(14.0)) / exp2f(20.0),
            p2: (f32::from(i16::from_le_bytes([value[7], value[8]])) - exp2f(14.0)) / exp2f(29.0),
            p3: f32::from(i8::from_le_bytes([value[9]])) / exp2f(32.0),
            p4: f32::from(i8::from_le_bytes([value[10]])) / exp2f(37.0),
            p5: f32::from(u16::from_le_bytes([value[11], value[12]])) / exp2f(-3.0),
            p6: f32::from(u16::from_le_bytes([value[13], value[14]])) / exp2f(6.0),
            p7: f32::from(i8::from_le_bytes([value[15]])) / exp2f(8.0),
            p8: f32::from(i8::from_le_bytes([value[16]])) / exp2f(15.0),
            p9: f32::from(i16::from_le_bytes([value[17], value[18]])) / exp2f(48.0),
            p10: f32::from(i8::from_le_bytes([value[19]])) / exp2f(48.0),
            p11: f32::from(i8::from_le_bytes([value[20]])) / exp2f(65.0),
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
}
pub struct BMP390ID {
    pub chip_id: u8,
    pub rev_id: u8
}

impl<I> BMP390Common<I>
where
    I: Interface,
{
    fn init<D: DelayNs>(
        &mut self,
        delay: &mut D,
        config: Option<Bmp390Config>,
    ) -> Result<BMP390ID, I::Error> {
        self.soft_reset(delay)?;
        let chip_id = self.read_chip_id()?;
        let rev_id = self.read_revision_id()?;
        match config {
            Some(v) => self.set_all_configs(&v)?,
            None => self.set_all_configs(&Bmp390Config::default())?,
        }
        Ok(BMP390ID {
            chip_id,
            rev_id
        })
    }

    fn read_chip_id(&mut self) -> Result<u8, I::Error> {
        self.interface.read_register(BMP390_CHIP_ID_REGISTER)
    }

    fn read_revision_id(&mut self) -> Result<u8, I::Error> {
        self.interface.read_register(BMP390_REV_ID_REGISTER)
    }

    fn soft_reset<D: DelayNs>(&mut self, delay: &mut D) -> Result<(), I::Error> {
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

    fn compensate_temperature(
        &self,
        raw_temperature: u32,
        compensation_data: &CompensationData,
    ) -> f32 {
        let comp_temp: f32;

        let partial_data1: f32;
        let partial_data2: f32;

        partial_data1 = raw_temperature as f32 - compensation_data.t1;
        partial_data2 = partial_data1 * compensation_data.t2;
        comp_temp = partial_data2 + (partial_data1 * partial_data1) * compensation_data.t3;

        comp_temp
    }

    fn compensate_pressure(
        &self,
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

        partial_data1 = compensation_data.p6 * compensated_temperature;
        partial_data2 = compensation_data.p7 * (compensated_temperature * compensated_temperature);
        partial_data3 = compensation_data.p8
            * (compensated_temperature * compensated_temperature * compensated_temperature);
        partial_out1 = compensation_data.p5 + partial_data1 + partial_data2 + partial_data3;

        partial_data1 = compensation_data.p2 * compensated_temperature;
        partial_data2 = compensation_data.p3 * (compensated_temperature * compensated_temperature);
        partial_data3 = compensation_data.p4
            * (compensated_temperature * compensated_temperature * compensated_temperature);
        partial_out2 = raw_pressure as f32
            * (compensation_data.p1 + partial_data1 + partial_data2 + partial_data3);

        partial_data1 = raw_pressure as f32 * raw_pressure as f32;
        partial_data2 = compensation_data.p9 + compensation_data.p10 * compensated_temperature;
        partial_data3 = partial_data1 * partial_data2;
        partial_data4 = partial_data3
            + (raw_pressure as f32 * raw_pressure as f32 * raw_pressure as f32 )
            * compensation_data.p11;

        comp_press = partial_out1 + partial_out2 + partial_data4;

        comp_press
    }

    fn take_measurement(&mut self) -> Result<BMP390Measurement, I::Error> {
        let raw_temperature = self.read_raw_temperature_data()?;
        let raw_pressure = self.read_raw_pressure_data()?;
        let compensation_data = self.read_compensation_data()?;

        let compensated_temperature =
            self.compensate_temperature(raw_temperature, &compensation_data);
        let compensated_pressure =
            self.compensate_pressure(raw_pressure, compensated_temperature, &compensation_data);

        Ok(BMP390Measurement {
            temp: compensated_temperature,
            press: compensated_pressure,
        })
    }

}
