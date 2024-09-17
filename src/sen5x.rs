use embedded_hal::{delay::DelayNs, i2c::I2c};
use sensirion_i2c::i2c as sen_i2c;

use crate::commands::Command;
use crate::crc;
use crate::types::{Sen5xData, Sen5xDataRaw};
use crate::Error;

/// The default I²C address of the SEN5X sensor.
const _SEN5X_I2C_ADDRESS: u8 = 0x69;

/// SEN5x sensor instance. Use related methods to take measurements.
#[derive(Debug, Default)]
pub struct Sen5x<I2C, D> {
    /// The concrete I²C device implementation.
    i2c: I2C,
    /// The concrete Delay implementation.
    delay: D,
    /// Whether the air quality measurement was initialized.
    is_running: bool,
    /// The I2C address of the sensor.
    address: u8,
}

impl<I2C, D, E> Sen5x<I2C, D>
where
    I2C: I2c<Error = E>,
    D: DelayNs,
{
    /// Create a new instance using the default I2C address.
    pub fn new(i2c: I2C, delay: D) -> Self {
        Self {
            i2c,
            delay,
            is_running: false,
            address: _SEN5X_I2C_ADDRESS,
        }
    }

    /// Create a new instance using a custom I2C address.
    pub fn with_i2c_address(i2c: I2C, delay: D, address: u8) -> Self {
        Self {
            i2c,
            delay,
            is_running: false,
            address,
        }
    }

    /// Start periodic measurement, signal update interval is 1 second.
    pub fn start_measurement(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartMeasurement)?;
        self.is_running = true;
        Ok(())
    }

    /// The reinit command reinitializes the sensor by reloading user settings from EEPROM.
    pub fn reinit(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::Reinit)?;
        Ok(())
    }

    /// Get 48-bit serial number.
    pub fn serial_number(&mut self) -> Result<u64, Error<E>> {
        let mut buf = [0; 9];
        self.delayed_read_cmd(Command::GetSerialNumber, &mut buf)?;
        let serial = u64::from(buf[0]) << 40
            | u64::from(buf[1]) << 32
            | u64::from(buf[3]) << 24
            | u64::from(buf[4]) << 16
            | u64::from(buf[6]) << 8
            | u64::from(buf[7]);

        Ok(serial)
    }

    /// Get 48-bit serial number.
    pub fn product_name(&mut self) -> Result<[u8; 32], Error<E>> {
        let mut buf = [0; 48];
        self.delayed_read_cmd(Command::ReadProductName, &mut buf)?;

        let mut bytes = [0u8; 32];
        for i in 0..16 {
            let hi = buf[i * 3 + 0];
            let lo = buf[i * 3 + 1];
            let crc = buf[i * 3 + 2];
            if crc::crc(&[hi, lo]) != crc {
                return Err(Error::Crc);
            }
            bytes[i * 2 + 0] = hi;
            bytes[i * 2 + 1] = lo;
        }

        Ok(bytes)
    }

    /// Read firmware version.
    pub fn read_firmware_version(&mut self) -> Result<u8, Error<E>> {
        let mut buf = [0u8; 3];
        self.delayed_read_cmd(Command::ReadFirmwareVersion, &mut buf)?;
        let [fw, res, crc] = buf;
        if crc::crc(&[fw, res]) != crc {
            return Err(Error::Crc);
        }
        Ok(fw)
    }

    /// Read raw sensor data.
    pub fn measurement_raw(&mut self) -> Result<Sen5xDataRaw, Error<E>> {
        let mut buf = [0; 24];
        self.delayed_read_cmd(Command::ReadMeasurement, &mut buf)?;

        let mut values = [0u16; 8];
        for value_idx in 0..8 {
            let hi = buf[value_idx * 3 + 0];
            let lo = buf[value_idx * 3 + 1];
            let crc = buf[value_idx * 3 + 2];
            if crc::crc(&[hi, lo]) != crc {
                return Err(Error::Crc);
            }
            values[value_idx] = u16::from_be_bytes([hi, lo]);
        }

        Ok(Sen5xDataRaw {
            pm1_0: values[0],
            pm2_5: values[1],
            pm4_0: values[2],
            pm10_0: values[3],
            humidity: values[4],
            temperature: values[5],
            voc_index: values[6],
            nox_index: values[7],
        })
    }

    /// Read converted sensor data.
    pub fn measurement(&mut self) -> Result<Sen5xData, Error<E>> {
        let data = self.measurement_raw()?;
        Ok(Sen5xData {
            pm1_0: data.pm1_0 as f32 / 10f32,
            pm2_5: data.pm2_5 as f32 / 10f32,
            pm4_0: data.pm4_0 as f32 / 10f32,
            pm10_0: data.pm10_0 as f32 / 10f32,
            temperature: data.temperature as f32 / 200f32,
            humidity: data.humidity as f32 / 100f32,
            voc_index: data.voc_index as f32 / 10f32,
            nox_index: data.nox_index as f32 / 10f32,
        })
    }

    /// Check whether new measurement data is available for read-out.
    pub fn data_ready_status(&mut self) -> Result<bool, Error<E>> {
        let mut buf = [0; 3];
        self.delayed_read_cmd(Command::GetReadDataReadyStatus, &mut buf)?;
        let status = u16::from_be_bytes([buf[0], buf[1]]);

        // 7FF is the last 11 bytes. If they are all zeroes, then data isn't ready.
        let ready = (status & 0x7FF) != 0;
        Ok(ready)
    }

    /// Writes commands without additional arguments.
    fn write_command(&mut self, cmd: Command) -> Result<(), Error<E>> {
        let (command, delay, _allowed_if_running) = cmd.as_tuple();
        sen_i2c::write_command_u16(&mut self.i2c, self.address, command).map_err(Error::I2c)?;
        self.delay.delay_ms(delay);
        Ok(())
    }

    /// Command for reading values from the sensor.
    fn delayed_read_cmd(&mut self, cmd: Command, data: &mut [u8]) -> Result<(), Error<E>> {
        self.write_command(cmd)?;
        let _ = sen_i2c::read_words_with_crc(&mut self.i2c, self.address, data).map_err(Error::I2c);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock as hal;

    use self::hal::eh1::delay::NoopDelay as DelayMock;
    use self::hal::eh1::i2c::{Mock as I2cMock, Transaction};
    use super::*;

    /// Test the get_serial_number function
    #[test]
    fn test_get_serial_number() {
        // Arrange
        let (cmd, _, _) = Command::GetSerialNumber.as_tuple();
        let expectations = [
            Transaction::write(_SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(
                _SEN5X_I2C_ADDRESS,
                vec![0xbe, 0xef, 0x92, 0xbe, 0xef, 0x92, 0xbe, 0xef, 0x92],
            ),
        ];
        let mut mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock.clone(), DelayMock);
        // Act
        let serial = sensor.serial_number().unwrap();
        // Assert
        assert_eq!(serial, 0xbeefbeefbeef);
        mock.done();
    }

    /// Test the measurement function
    #[test]
    fn test_measurement() {
        // Arrange
        let (cmd, _, _) = Command::ReadMeasurement.as_tuple();
        let expectations = [
            Transaction::write(_SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(
                _SEN5X_I2C_ADDRESS,
                vec![
                    0x00, 0x12, 0xA0, 0x00, 0x16, 0x64, 0x00, 0x18, 0x7B, 0x00, 0x1A, 0x19, 0x15,
                    0x8A, 0x39, 0x11, 0x81, 0x50, 0x01, 0x68, 0x77, 0x00, 0x0A, 0x5A,
                ],
            ),
        ];
        let mut mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock.clone(), DelayMock);
        // Act
        let data = sensor.measurement().unwrap();
        // Assert
        assert_eq!(data.pm2_5, 2.200_f32);
        assert_eq!(data.temperature, 22.405_f32);
        assert_eq!(data.humidity, 55.14_f32);
        mock.done()
    }
}
