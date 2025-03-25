// SPDX-FileCopyrightText: Joonas Javanainen <joonas@merulogic.com>
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Low-level register definitions
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[cfg(feature = "defmt-03")]
use crate::defmt;

use super::Mode;

/// Register address
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum RegisterAddress {
    /// Company ID
    Wia1 = 0x00,
    /// Device ID
    Wia2 = 0x01,
    /// Reserved 1
    Rsv1 = 0x02,
    /// Reserved 2
    Rsv2 = 0x03,
    /// Status 1
    St1 = 0x10,
    /// Measurement Magnetic Data (X axis, LSB)
    Hxl = 0x11,
    /// Measurement Magnetic Data (X axis, MSB)
    Hxh = 0x12,
    /// Measurement Magnetic Data (Y axis, LSB)
    Hyl = 0x13,
    /// Measurement Magnetic Data (Y axis, MSB)
    Hyh = 0x14,
    /// Measurement Magnetic Data (Z axis, LSB)
    Hzl = 0x15,
    /// Measurement Magnetic Data (Z axis, MSB)
    Hzh = 0x16,
    /// Dummy
    Tmps = 0x17,
    /// Status 2
    St2 = 0x18,
    /// Dummy
    Cntl1 = 0x30,
    /// Control 2
    Cntl2 = 0x31,
    /// Control 3
    Cntl3 = 0x32,
    /// Test (DO NOT ACCESS)
    Ts1 = 0x33,
    /// Test (DO NOT ACCESS)
    Ts2 = 0x34,
}

#[cfg(feature = "defmt-03")]
impl defmt::Format for RegisterAddress {
    fn format(&self, fmt: defmt::Formatter) {
        u8::from(*self).format(fmt)
    }
}

/// 8-bit register
pub trait Register8: From<u8> + Into<u8> {
    const ADDRESS: RegisterAddress;
}

/// 16-bit register with signed two's complement data
pub trait Register16: From<i16> + Into<i16> {
    const ADDRESS: RegisterAddress;
}

macro_rules! impl_transparent_reg8 {
    ($name:tt, $addr:expr) => {
        impl crate::regs::Register8 for $name {
            const ADDRESS: RegisterAddress = $addr;
        }

        impl From<u8> for $name {
            fn from(value: u8) -> Self {
                $name(value)
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

macro_rules! impl_bitflags_reg8 {
    ($name:tt, $addr:expr) => {
        impl crate::regs::Register8 for $name {
            const ADDRESS: RegisterAddress = $addr;
        }

        impl From<u8> for $name {
            fn from(value: u8) -> Self {
                $name::from_bits_truncate(value)
            }
        }

        impl From<$name> for u8 {
            fn from(value: $name) -> Self {
                value.bits()
            }
        }
    };
}

/// Who I Am 1 (Company ID)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Wia1(
    /// Company ID
    pub u8,
);

impl Wia1 {
    /// Company ID of AKM
    pub const AKM: Wia1 = Wia1(0x48);
}

impl_transparent_reg8!(Wia1, RegisterAddress::Wia1);

/// Who I Am 2 (Device ID)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Wia2(
    /// Device ID
    pub u8,
);

impl Wia2 {
    /// Device ID of AK09916
    pub const AK09916: Wia2 = Wia2(0x09);
}

impl_transparent_reg8!(Wia2, RegisterAddress::Wia2);

#[cfg(not(feature = "defmt-03"))]
bitflags::bitflags! {
    /// Status 1
    #[repr(transparent)]
    pub struct St1: u8 {
        /// Data Overrun
        const DOR = 1 << 1;
        /// Data Ready
        const DRDY = 1 << 0;
    }
}

#[cfg(feature = "defmt-03")]
defmt::bitflags! {
    /// Status 1
    #[repr(transparent)]
    pub struct St1: u8 {
        /// Data Overrun
        const DOR = 1 << 1;
        /// Data Ready
        const DRDY = 1 << 0;
    }
}

impl_bitflags_reg8!(St1, RegisterAddress::St1);

/// Measurement Magnetic Data (X axis, LSB)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hxl(
    /// X-axis measurement data (LSB)
    pub u8,
);

impl_transparent_reg8!(Hxl, RegisterAddress::Hxl);

/// Measurement Magnetic Data (X axis, MSB)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hxh(
    /// X-axis measurement data (MSB)
    pub u8,
);

impl_transparent_reg8!(Hxh, RegisterAddress::Hxh);

/// Measurement Magnetic Data (X axis)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hx(
    /// X-axis measurement data
    pub i16,
);

impl Register16 for Hx {
    const ADDRESS: RegisterAddress = RegisterAddress::Hxl;
}

impl From<i16> for Hx {
    fn from(value: i16) -> Self {
        Hx(value)
    }
}

impl From<Hx> for i16 {
    fn from(value: Hx) -> Self {
        value.0
    }
}

/// Measurement Magnetic Data (Y axis, LSB)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hyl(
    /// Y-axis measurement data (LSB)
    pub u8,
);

impl_transparent_reg8!(Hyl, RegisterAddress::Hyl);

/// Measurement Magnetic Data (Y axis, MSB)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hyh(
    /// Y-axis measurement data (MSB)
    pub u8,
);

impl_transparent_reg8!(Hyh, RegisterAddress::Hyh);

/// Measurement Magnetic Data (Y axis)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hy(
    /// Y-axis measurement data
    pub i16,
);

impl Register16 for Hy {
    const ADDRESS: RegisterAddress = RegisterAddress::Hyl;
}

impl From<i16> for Hy {
    fn from(value: i16) -> Self {
        Hy(value)
    }
}

impl From<Hy> for i16 {
    fn from(value: Hy) -> Self {
        value.0
    }
}

/// Measurement Magnetic Data (Z axis, LSB)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hzl(
    /// Z-axis measurement data (LSB)
    pub u8,
);

impl_transparent_reg8!(Hzl, RegisterAddress::Hzl);

/// Measurement Magnetic Data (Z axis, MSB)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hzh(
    /// Z-axis measurement data (MSB)
    pub u8,
);

impl_transparent_reg8!(Hzh, RegisterAddress::Hzh);

/// Measurement Magnetic Data (Z axis)
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Hz(
    /// Z-axis measurement data
    pub i16,
);

impl Register16 for Hz {
    const ADDRESS: RegisterAddress = RegisterAddress::Hzl;
}

impl From<i16> for Hz {
    fn from(value: i16) -> Self {
        Hz(value)
    }
}

impl From<Hz> for i16 {
    fn from(value: Hz) -> Self {
        value.0
    }
}

#[cfg(not(feature = "defmt-03"))]
bitflags::bitflags! {
    /// Status 2
    #[repr(transparent)]
    pub struct St2: u8 {
        /// Reserved
        const RSV30 = 1 << 6;
        /// Reserved
        const RSV29 = 1 << 5;
        /// Reserved
        const RSV28 = 1 << 4;
        /// Magnetic sensor overflow
        const HOFL = 1 << 3;
    }
}

#[cfg(feature = "defmt-03")]
defmt::bitflags! {
    /// Status 2
    #[repr(transparent)]
    pub struct St2: u8 {
        /// Reserved
        const RSV30 = 1 << 6;
        /// Reserved
        const RSV29 = 1 << 5;
        /// Reserved
        const RSV28 = 1 << 4;
        /// Magnetic sensor overflow
        const HOFL = 1 << 3;
    }
}

impl Register8 for St2 {
    const ADDRESS: RegisterAddress = RegisterAddress::St2;
}

impl From<u8> for St2 {
    fn from(value: u8) -> Self {
        St2::from_bits_truncate(value)
    }
}

impl From<St2> for u8 {
    fn from(value: St2) -> Self {
        value.bits()
    }
}

/// Operation mode setting
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum ModeRegister {
    Mode(Mode),
    Other(u8),
}

impl From<u8> for ModeRegister {
    fn from(value: u8) -> Self {
        match Mode::try_from(value & 0b11111) {
            Ok(mode) => ModeRegister::Mode(mode),
            Err(err) => ModeRegister::Other(err.number),
        }
    }
}

impl From<ModeRegister> for u8 {
    fn from(value: ModeRegister) -> Self {
        match value {
            ModeRegister::Mode(mode) => u8::from(mode),
            ModeRegister::Other(value) => value,
        }
    }
}

/// Control 2
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Cntl2(
    /// Operation mode setting
    pub ModeRegister,
);

impl From<Mode> for Cntl2 {
    fn from(value: Mode) -> Self {
        Cntl2(ModeRegister::Mode(value))
    }
}

impl Register8 for Cntl2 {
    const ADDRESS: RegisterAddress = RegisterAddress::Cntl2;
}

impl From<u8> for Cntl2 {
    fn from(value: u8) -> Self {
        Cntl2(ModeRegister::from(value))
    }
}

impl From<Cntl2> for u8 {
    fn from(value: Cntl2) -> Self {
        u8::from(value.0)
    }
}

#[cfg(not(feature = "defmt-03"))]
bitflags::bitflags! {
    /// Control 3
    #[repr(transparent)]
    pub struct Cntl3: u8 {
        /// Soft reset
        const SRST = 1 << 0;
    }
}

#[cfg(feature = "defmt-03")]
defmt::bitflags! {
    /// Control 3
    #[repr(transparent)]
    pub struct Cntl3: u8 {
        /// Soft reset
        const SRST = 1 << 0;
    }
}

impl Register8 for Cntl3 {
    const ADDRESS: RegisterAddress = RegisterAddress::Cntl3;
}

impl From<u8> for Cntl3 {
    fn from(value: u8) -> Self {
        Cntl3::from_bits_truncate(value)
    }
}

impl From<Cntl3> for u8 {
    fn from(value: Cntl3) -> Self {
        value.bits()
    }
}

/// Full dump of non-reserved registers and their bits
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct RegisterDump {
    pub company_id: Wia1,
    pub device_id: Wia2,
    pub st1: St1,
    pub hx: i16,
    pub hy: i16,
    pub hz: i16,
    pub st2: St2,
    pub mode: ModeRegister,
    pub cntl3: Cntl3,
}

impl RegisterDump {
    #[inline]
    pub(crate) fn from_raw_data(buffer: [u8; 16]) -> Self {
        RegisterDump {
            company_id: Wia1::from(buffer[0]),
            device_id: Wia2::from(buffer[1]),
            st1: St1::from(buffer[4]),
            hx: i16::from_le_bytes([buffer[5], buffer[6]]),
            hy: i16::from_le_bytes([buffer[7], buffer[8]]),
            hz: i16::from_le_bytes([buffer[9], buffer[10]]),
            st2: St2::from(buffer[12]),
            mode: Cntl2::from(buffer[14]).0,
            cntl3: Cntl3::from(buffer[15]),
        }
    }
}
