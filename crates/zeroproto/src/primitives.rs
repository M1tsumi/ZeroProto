//! Primitive type handling and endian utilities

use crate::errors::Result;

/// Endianness for byte order handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    /// Little-endian byte order
    Little,
    /// Big-endian byte order
    Big,
}

/// Primitive types supported by ZeroProto
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PrimitiveType {
    /// 8-bit unsigned integer
    U8 = 0,
    /// 16-bit unsigned integer
    U16 = 1,
    /// 32-bit unsigned integer
    U32 = 2,
    /// 64-bit unsigned integer
    U64 = 3,
    /// 8-bit signed integer
    I8 = 4,
    /// 16-bit signed integer
    I16 = 5,
    /// 32-bit signed integer
    I32 = 6,
    /// 64-bit signed integer
    I64 = 7,
    /// 32-bit floating point
    F32 = 8,
    /// 64-bit floating point
    F64 = 9,
    /// Boolean value
    Bool = 10,
    /// UTF-8 string
    String = 11,
    /// Byte slice
    Bytes = 12,
    /// Nested message
    Message = 13,
    /// Vector of values
    Vector = 14,
    /// Sentinel for unset/absent fields
    Unset = 255,
}

impl PrimitiveType {
    /// Get the size of this primitive type in bytes
    pub fn size(self) -> Option<usize> {
        match self {
            PrimitiveType::U8 => Some(1),
            PrimitiveType::U16 => Some(2),
            PrimitiveType::U32 => Some(4),
            PrimitiveType::U64 => Some(8),
            PrimitiveType::I8 => Some(1),
            PrimitiveType::I16 => Some(2),
            PrimitiveType::I32 => Some(4),
            PrimitiveType::I64 => Some(8),
            PrimitiveType::F32 => Some(4),
            PrimitiveType::F64 => Some(8),
            PrimitiveType::Bool => Some(1),
            PrimitiveType::String
            | PrimitiveType::Bytes
            | PrimitiveType::Message
            | PrimitiveType::Vector
            | PrimitiveType::Unset => None,
        }
    }

    /// Convert from byte to PrimitiveType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(PrimitiveType::U8),
            1 => Some(PrimitiveType::U16),
            2 => Some(PrimitiveType::U32),
            3 => Some(PrimitiveType::U64),
            4 => Some(PrimitiveType::I8),
            5 => Some(PrimitiveType::I16),
            6 => Some(PrimitiveType::I32),
            7 => Some(PrimitiveType::I64),
            8 => Some(PrimitiveType::F32),
            9 => Some(PrimitiveType::F64),
            10 => Some(PrimitiveType::Bool),
            11 => Some(PrimitiveType::String),
            12 => Some(PrimitiveType::Bytes),
            13 => Some(PrimitiveType::Message),
            14 => Some(PrimitiveType::Vector),
            255 => Some(PrimitiveType::Unset),
            _ => None,
        }
    }
}

impl Endian {
    /// Read a u8 value
    #[inline]
    pub fn read_u8(self, buf: &[u8], offset: usize) -> u8 {
        buf[offset]
    }

    /// Read a u16 value
    #[inline]
    pub fn read_u16(self, buf: &[u8], offset: usize) -> u16 {
        let bytes = &buf[offset..offset + 2];
        match self {
            Endian::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
            Endian::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
        }
    }

    /// Read a u32 value
    #[inline]
    pub fn read_u32(self, buf: &[u8], offset: usize) -> u32 {
        let bytes = &buf[offset..offset + 4];
        match self {
            Endian::Little => u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            Endian::Big => u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        }
    }

    /// Read a u64 value
    #[inline]
    pub fn read_u64(self, buf: &[u8], offset: usize) -> u64 {
        let bytes = &buf[offset..offset + 8];
        match self {
            Endian::Little => u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            Endian::Big => u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
        }
    }

    /// Read an i8 value
    #[inline]
    pub fn read_i8(self, buf: &[u8], offset: usize) -> i8 {
        self.read_u8(buf, offset) as i8
    }

    /// Read an i16 value
    #[inline]
    pub fn read_i16(self, buf: &[u8], offset: usize) -> i16 {
        self.read_u16(buf, offset) as i16
    }

    /// Read an i32 value
    #[inline]
    pub fn read_i32(self, buf: &[u8], offset: usize) -> i32 {
        self.read_u32(buf, offset) as i32
    }

    /// Read an i64 value
    #[inline]
    pub fn read_i64(self, buf: &[u8], offset: usize) -> i64 {
        self.read_u64(buf, offset) as i64
    }

    /// Read an f32 value
    #[inline]
    pub fn read_f32(self, buf: &[u8], offset: usize) -> f32 {
        f32::from_bits(self.read_u32(buf, offset))
    }

    /// Read an f64 value
    #[inline]
    pub fn read_f64(self, buf: &[u8], offset: usize) -> f64 {
        f64::from_bits(self.read_u64(buf, offset))
    }

    /// Read a bool value
    #[inline]
    pub fn read_bool(self, buf: &[u8], offset: usize) -> bool {
        self.read_u8(buf, offset) != 0
    }

    /// Write a u8 value
    #[inline]
    pub fn write_u8(self, value: u8, buf: &mut [u8], offset: usize) {
        buf[offset] = value;
    }

    /// Write a u16 value
    #[inline]
    pub fn write_u16(self, value: u16, buf: &mut [u8], offset: usize) {
        let bytes = match self {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        buf[offset..offset + 2].copy_from_slice(&bytes);
    }

    /// Write a u32 value
    #[inline]
    pub fn write_u32(self, value: u32, buf: &mut [u8], offset: usize) {
        let bytes = match self {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        buf[offset..offset + 4].copy_from_slice(&bytes);
    }

    /// Write a u64 value
    #[inline]
    pub fn write_u64(self, value: u64, buf: &mut [u8], offset: usize) {
        let bytes = match self {
            Endian::Little => value.to_le_bytes(),
            Endian::Big => value.to_be_bytes(),
        };
        buf[offset..offset + 8].copy_from_slice(&bytes);
    }

    /// Write an i8 value
    #[inline]
    pub fn write_i8(self, value: i8, buf: &mut [u8], offset: usize) {
        self.write_u8(value as u8, buf, offset);
    }

    /// Write an i16 value
    #[inline]
    pub fn write_i16(self, value: i16, buf: &mut [u8], offset: usize) {
        self.write_u16(value as u16, buf, offset);
    }

    /// Write an i32 value
    #[inline]
    pub fn write_i32(self, value: i32, buf: &mut [u8], offset: usize) {
        self.write_u32(value as u32, buf, offset);
    }

    /// Write an i64 value
    #[inline]
    pub fn write_i64(self, value: i64, buf: &mut [u8], offset: usize) {
        self.write_u64(value as u64, buf, offset);
    }

    /// Write an f32 value
    #[inline]
    pub fn write_f32(self, value: f32, buf: &mut [u8], offset: usize) {
        self.write_u32(value.to_bits(), buf, offset);
    }

    /// Write an f64 value
    #[inline]
    pub fn write_f64(self, value: f64, buf: &mut [u8], offset: usize) {
        self.write_u64(value.to_bits(), buf, offset);
    }

    /// Write a bool value
    #[inline]
    pub fn write_bool(self, value: bool, buf: &mut [u8], offset: usize) {
        self.write_u8(value as u8, buf, offset);
    }
}
