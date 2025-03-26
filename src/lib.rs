// SPDX-FileCopyrightText: Joonas Javanainen <joonas@merulogic.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Driver for the AKM AK09916 3-axis I²C magnetometer
//!
//! Example (blocking API):
//!
//! ```rust
//! use ak09916::{blocking::Ak09916, Mode, WhoIAm};
//! use embedded_hal::{delay::DelayNs, i2c::I2c};
//! // This example assumes something like defmt is available for logging:
//! // use defmt::info;
//! #
//! # // Dummy macro
//! # macro_rules! info {
//! #    ($($arg:tt)+) => ()
//! # }
//!
//! fn example<I: I2c, D: DelayNs>(i2c: I, delay: D) -> Result<(), I::Error> {
//!     let mut ak09916 = Ak09916::new(i2c, delay);
//!
//!     // optional: check who I am (WIA) information
//!     let wia = ak09916.who_i_am()?;
//!     if wia != WhoIAm::AK09916 {
//!       // try again or fail, depending on your use case
//!     }
//!
//!     // optional: do a self-test
//!     let test_result = ak09916.self_test()?;
//!     if !test_result.is_valid {
//!       // try again or fail, depending on your use case
//!     }
//!
//!     ak09916.switch_mode(Mode::Continuous10Hz)?;
//!     loop {
//!         // In continuous 10 Hz measurement mode we get a measurement every 100ms, so the poll
//!         // interval needs to be much less than that or we might miss measurements
//!         const POLL_INTERVAL_US: u32 = 10_000; // 10 ms
//!
//!         let measurement = ak09916.poll_measurement(POLL_INTERVAL_US)?;
//!         if measurement.overflow() {
//!             info!("Magnetic sensor overflow: data is not valid");
//!         }
//!         if measurement.overrun() {
//!             info!("Data overrun: at least one measurement was missed");
//!         }
//!         info!("X-axis: {} nT", measurement.x_nanoteslas());
//!         info!("Y-axis: {} nT", measurement.y_nanoteslas());
//!         info!("Z-axis: {} nT", measurement.z_nanoteslas());
//!     }
//! }
//! ```

#![no_std]

#[cfg(feature = "defmt-03")]
use defmt_03 as defmt;

#[cfg(not(feature = "defmt-03"))]
use bitflags::bitflags as bitflags_macro;

#[cfg(feature = "defmt-03")]
use crate::defmt::bitflags as bitflags_macro;

pub mod regs;

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// I²C address of AK09916
pub const I2C_ADDRESS: u8 = 0x0c;
/// Minimum wait time before setting mode in μs
///
/// High-level driver functions like [`switch_mode`](blocking::Ak09916::switch_mode) automatically
/// use this, so this is only needed if you do mode switches with low-level functions like
/// [`write_register8`](blocking::Ak09916::write_register8).
pub const MODE_SET_WAIT_TIME_US: u32 = 100;
/// Sensitivity of the sensor as nT / bit.
///
/// This can be used to convert the raw measurement `hx` / `hy` / `hz` values to nanoteslas (nT).
pub const SENSITIVITY_NT_PER_BIT: i32 = 150;

/// Who I Am register data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct WhoIAm {
    /// Company ID
    pub company_id: u8,
    /// Device ID
    pub device_id: u8,
}

impl WhoIAm {
    /// Expected Who I Am data for AK09916
    pub const AK09916: WhoIAm = WhoIAm {
        company_id: regs::Wia1::AKM.0,
        device_id: regs::Wia2::AK09916.0,
    };
}

/// Operation mode setting
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum Mode {
    /// Power-down mode
    PowerDown = 0b00000,
    /// Single measurement mode
    SingleMeasurement = 0b00001,
    /// Continuous measurement mode 1 (10 Hz)
    Continuous10Hz = 0b00010,
    /// Continuous measurement mode 2 (20 Hz)
    Continuous20Hz = 0b00100,
    /// Continuous measurement mode 3 (50 Hz)
    Continuous50Hz = 0b00110,
    /// Continuous measurement mode 4 (100 Hz)
    Continuous100Hz = 0b01000,
    /// Self-test mode
    SelfTest = 0b10000,
}

impl Mode {
    /// Alias for [`Mode::Continuous10Hz`]
    pub const CONTINUOUS_1: Mode = Mode::Continuous10Hz;
    /// Alias for [`Mode::Continuous20Hz`]
    pub const CONTINUOUS_2: Mode = Mode::Continuous20Hz;
    /// Alias for [`Mode::Continuous50Hz`]
    pub const CONTINUOUS_3: Mode = Mode::Continuous50Hz;
    /// Alias for [`Mode::Continuous100Hz`]
    pub const CONTINUOUS_4: Mode = Mode::Continuous100Hz;
}

/// Measurement data
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Measurement {
    /// X-axis (raw value)
    pub hx: i16,
    /// Y-axis (raw value)
    pub hy: i16,
    /// Z-axis (raw value)
    pub hz: i16,
    /// Flags
    pub flags: MeasurementFlags,
}

impl Measurement {
    /// X-axis (in nT)
    pub fn x_nanoteslas(&self) -> i32 {
        i32::from(self.hx) * SENSITIVITY_NT_PER_BIT
    }
    /// Y-axis (in nT)
    pub fn y_nanoteslas(&self) -> i32 {
        i32::from(self.hy) * SENSITIVITY_NT_PER_BIT
    }
    /// Z-axis (in nT)
    pub fn z_nanoteslas(&self) -> i32 {
        i32::from(self.hz) * SENSITIVITY_NT_PER_BIT
    }
    /// Returns true if flags indicate data overrun has happened
    pub fn overrun(&self) -> bool {
        self.flags.contains(MeasurementFlags::OVERRUN)
    }
    /// Returns true if flags indicate magnetic sensor overflow has happened
    pub fn overflow(&self) -> bool {
        self.flags.contains(MeasurementFlags::OVERFLOW)
    }
    #[inline]
    fn from_raw_data(st1: regs::St1, buffer: [u8; 8]) -> Measurement {
        let st2 = regs::St2::from(buffer[7]);
        Measurement {
            hx: i16::from_le_bytes([buffer[0], buffer[1]]),
            hy: i16::from_le_bytes([buffer[2], buffer[3]]),
            hz: i16::from_le_bytes([buffer[4], buffer[5]]),
            flags: if st1.contains(regs::St1::DOR) {
                MeasurementFlags::OVERRUN
            } else {
                MeasurementFlags::empty()
            } | if st2.contains(regs::St2::HOFL) {
                MeasurementFlags::OVERFLOW
            } else {
                MeasurementFlags::empty()
            },
        }
    }
}

bitflags_macro! {
    /// Measurement flags
    #[repr(transparent)]
    pub struct MeasurementFlags: u8 {
        /// Magnetic sensor overflow
        const OVERFLOW = 1 << 3;
        /// Data overrun
        const OVERRUN = 1 << 1;
    }
}

/// Result for a self-test
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct SelfTestResult {
    /// Measurement data
    pub measurement: Measurement,
    /// True if self-test measurement data is valid and the test is passed
    pub is_valid: bool,
}

impl From<Measurement> for SelfTestResult {
    fn from(measurement: Measurement) -> Self {
        let is_valid = (-200..=200).contains(&measurement.hx)
            && (-200..=200).contains(&measurement.hy)
            && (-1000..=-200).contains(&measurement.hz);
        Self {
            measurement,
            is_valid,
        }
    }
}

/// Asynchronous API
pub mod asynch {
    use embedded_hal_async::{delay::DelayNs, i2c::I2c};

    use crate::{
        regs::{self, Register16, Register8, RegisterAddress},
        Measurement, Mode, SelfTestResult, WhoIAm, I2C_ADDRESS, MODE_SET_WAIT_TIME_US,
    };

    /// AK09916 driver
    pub struct Ak09916<I: I2c, D: DelayNs> {
        i2c: I,
        delay: D,
    }

    impl<I: I2c, D: DelayNs> Ak09916<I, D> {
        /// Creates a new asynchronous AK09916 driver
        pub fn new(i2c: I, delay: D) -> Self {
            Ak09916 { i2c, delay }
        }
        /// Consumes the driver and releases resources used by it
        pub fn release(self) -> (I, D) {
            let Ak09916 { i2c, delay } = self;
            (i2c, delay)
        }
        /// Reads the Who I Am information from the device
        pub async fn who_i_am(&mut self) -> Result<WhoIAm, I::Error> {
            let mut buffer = [0, 0];
            self.i2c
                .write_read(I2C_ADDRESS, &[u8::from(RegisterAddress::Wia1)], &mut buffer)
                .await?;
            Ok(WhoIAm {
                company_id: regs::Wia1::from(buffer[0]).0,
                device_id: regs::Wia2::from(buffer[1]).0,
            })
        }
        /// Polls the device for measurement data until it's available
        pub async fn poll_measurement(
            &mut self,
            poll_interval_us: u32,
        ) -> Result<Measurement, I::Error> {
            let mut st1: regs::St1;
            loop {
                st1 = self.read_register8::<regs::St1>().await?;
                if st1.contains(regs::St1::DRDY) {
                    break;
                }
                self.delay.delay_us(poll_interval_us).await;
            }
            let mut buffer = [0; 8];
            self.i2c.read(I2C_ADDRESS, &mut buffer).await?;
            Ok(Measurement::from_raw_data(st1, buffer))
        }
        /// Reads the latest measurement data, if available.
        ///
        /// Returns None if measurement data is not ready
        pub async fn read_measurement(&mut self) -> Result<Option<Measurement>, I::Error> {
            let st1 = self.read_register8::<regs::St1>().await?;
            if st1.contains(regs::St1::DRDY) {
                let mut buffer = [0; 8];
                self.i2c.read(I2C_ADDRESS, &mut buffer).await?;
                Ok(Some(Measurement::from_raw_data(st1, buffer)))
            } else {
                Ok(None)
            }
        }
        /// Switches the device to the given mode
        pub async fn switch_mode(&mut self, target_mode: Mode) -> Result<(), I::Error> {
            self.write_register8(regs::Cntl2::from(Mode::PowerDown))
                .await?;
            self.delay.delay_us(MODE_SET_WAIT_TIME_US).await;
            self.write_register8(regs::Cntl2::from(target_mode)).await
        }
        /// Performs a self-test.
        ///
        /// The device switches to power-down mode automatically after the operation.
        pub async fn self_test(&mut self) -> Result<SelfTestResult, I::Error> {
            self.switch_mode(Mode::SelfTest).await?;
            let measurement = self.poll_measurement(10).await?;
            Ok(SelfTestResult::from(measurement))
        }
        /// Performs a soft-reset.
        ///
        /// The device switches to power-down mode automatically after the operation.
        pub async fn soft_reset(&mut self) -> Result<(), I::Error> {
            self.write_register8(regs::Cntl3::SRST).await?;
            loop {
                self.delay.delay_us(MODE_SET_WAIT_TIME_US).await;
                let cntl3 = self.read_register8::<regs::Cntl3>().await?;
                if !cntl3.contains(regs::Cntl3::SRST) {
                    break Ok(());
                }
            }
        }
    }

    /// Low-level register access API
    impl<I: I2c, D: DelayNs> Ak09916<I, D> {
        /// Reads an 8-bit register
        pub async fn read_register8<R: Register8>(&mut self) -> Result<R, I::Error> {
            let mut buffer = [0];
            self.i2c
                .write_read(I2C_ADDRESS, &[u8::from(R::ADDRESS)], &mut buffer)
                .await?;
            Ok(R::from(buffer[0]))
        }
        /// Reads a 16-bit register
        pub async fn read_register16<R: Register16>(&mut self) -> Result<R, I::Error> {
            let mut buffer = [0, 0];
            self.i2c
                .write_read(I2C_ADDRESS, &[u8::from(R::ADDRESS)], &mut buffer)
                .await?;
            Ok(R::from(i16::from_le_bytes(buffer)))
        }
        /// Writes a 8-bit register
        pub async fn write_register8<R: Register8>(&mut self, register: R) -> Result<(), I::Error> {
            let buffer = [u8::from(R::ADDRESS), register.into()];
            self.i2c.write(I2C_ADDRESS, &buffer).await
        }
        /// Dumps all non-reserved register data
        pub async fn dump_registers(&mut self) -> Result<regs::RegisterDump, I::Error> {
            let mut buffer = [0; 16];
            self.i2c
                .write_read(I2C_ADDRESS, &[u8::from(RegisterAddress::Wia1)], &mut buffer)
                .await?;
            Ok(regs::RegisterDump::from_raw_data(buffer))
        }
    }
}

/// Blocking API
pub mod blocking {
    use embedded_hal::{delay::DelayNs, i2c::I2c};

    use crate::{
        regs::{self, Register16, Register8, RegisterAddress},
        Measurement, Mode, SelfTestResult, WhoIAm, I2C_ADDRESS, MODE_SET_WAIT_TIME_US,
    };

    /// AK09916 driver
    pub struct Ak09916<I: I2c, D: DelayNs> {
        i2c: I,
        delay: D,
    }

    impl<I: I2c, D: DelayNs> Ak09916<I, D> {
        /// Creates a new blocking AK09916 driver
        pub fn new(i2c: I, delay: D) -> Self {
            Ak09916 { i2c, delay }
        }
        /// Consumes the driver and releases resources used by it
        pub fn release(self) -> (I, D) {
            let Ak09916 { i2c, delay } = self;
            (i2c, delay)
        }
        /// Reads the Who I Am information from the device
        pub fn who_i_am(&mut self) -> Result<WhoIAm, I::Error> {
            let mut buffer = [0, 0];
            self.i2c
                .write_read(I2C_ADDRESS, &[u8::from(RegisterAddress::Wia1)], &mut buffer)?;
            Ok(WhoIAm {
                company_id: regs::Wia1::from(buffer[0]).0,
                device_id: regs::Wia2::from(buffer[1]).0,
            })
        }
        /// Polls the device for measurement data until it's available
        pub fn poll_measurement(&mut self, poll_interval_us: u32) -> Result<Measurement, I::Error> {
            let mut st1: regs::St1;
            loop {
                st1 = self.read_register8::<regs::St1>()?;
                if st1.contains(regs::St1::DRDY) {
                    break;
                }
                self.delay.delay_us(poll_interval_us);
            }
            let mut buffer = [0; 8];
            self.i2c.read(I2C_ADDRESS, &mut buffer)?;
            Ok(Measurement::from_raw_data(st1, buffer))
        }
        /// Reads the latest measurement data, if available.
        ///
        /// Returns None if measurement data is not ready
        pub fn read_measurement(&mut self) -> Result<Option<Measurement>, I::Error> {
            let st1 = self.read_register8::<regs::St1>()?;
            if st1.contains(regs::St1::DRDY) {
                let mut buffer = [0; 8];
                self.i2c.read(I2C_ADDRESS, &mut buffer)?;
                Ok(Some(Measurement::from_raw_data(st1, buffer)))
            } else {
                Ok(None)
            }
        }
        /// Switches the device to the given mode
        pub fn switch_mode(&mut self, target_mode: Mode) -> Result<(), I::Error> {
            self.write_register8(regs::Cntl2::from(Mode::PowerDown))?;
            self.delay.delay_us(MODE_SET_WAIT_TIME_US);
            self.write_register8(regs::Cntl2::from(target_mode))
        }
        /// Performs a self-test.
        ///
        /// The device switches to power-down mode automatically after the operation.
        pub fn self_test(&mut self) -> Result<SelfTestResult, I::Error> {
            self.switch_mode(Mode::SelfTest)?;
            let measurement = self.poll_measurement(10)?;
            Ok(SelfTestResult::from(measurement))
        }
        /// Performs a soft-reset.
        ///
        /// The device switches to power-down mode automatically after the operation.
        pub fn soft_reset(&mut self) -> Result<(), I::Error> {
            self.write_register8(regs::Cntl3::SRST)?;
            loop {
                self.delay.delay_us(MODE_SET_WAIT_TIME_US);
                let cntl3 = self.read_register8::<regs::Cntl3>()?;
                if !cntl3.contains(regs::Cntl3::SRST) {
                    break Ok(());
                }
            }
        }
    }

    /// Low-level register access API
    impl<I: I2c, D: DelayNs> Ak09916<I, D> {
        /// Reads an 8-bit register
        pub fn read_register8<R: Register8>(&mut self) -> Result<R, I::Error> {
            let mut buffer = [0];
            self.i2c
                .write_read(I2C_ADDRESS, &[u8::from(R::ADDRESS)], &mut buffer)?;
            Ok(R::from(buffer[0]))
        }
        /// Reads a 16-bit register
        pub fn read_register16<R: Register16>(&mut self) -> Result<R, I::Error> {
            let mut buffer = [0, 0];
            self.i2c
                .write_read(I2C_ADDRESS, &[u8::from(R::ADDRESS)], &mut buffer)?;
            Ok(R::from(i16::from_le_bytes(buffer)))
        }
        /// Writes a 8-bit register
        pub fn write_register8<R: Register8>(&mut self, register: R) -> Result<(), I::Error> {
            let buffer = [u8::from(R::ADDRESS), register.into()];
            self.i2c.write(I2C_ADDRESS, &buffer)
        }
        /// Dumps all non-reserved register data
        pub fn dump_registers(&mut self) -> Result<regs::RegisterDump, I::Error> {
            let mut buffer = [0; 16];
            self.i2c
                .write_read(I2C_ADDRESS, &[u8::from(RegisterAddress::Wia1)], &mut buffer)?;
            Ok(regs::RegisterDump::from_raw_data(buffer))
        }
    }
}
