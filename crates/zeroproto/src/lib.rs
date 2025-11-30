//! ZeroProto - Zero-copy binary serialization runtime library
//!
//! This crate provides the runtime components for ZeroProto, including:
//! - Reader types for zero-copy deserialization
//! - Builder types for serialization
//! - Error handling and safety utilities
//! - Buffer abstractions
//!
//! # Quick Start
//!
//! ```rust
//! use zeroproto::{MessageReader, MessageBuilder};
//!
//! // Serialize
//! let mut builder = MessageBuilder::new();
//! builder.set_scalar(0, 42u64)?;
//! let data = builder.finish();
//!
//! // Deserialize (zero-copy)
//! let reader = MessageReader::new(&data)?;
//! let field: u64 = reader.get_scalar(0)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::fmt;

mod builder;
mod errors;
mod primitives;
mod reader;
mod vector;

pub use builder::{MessageBuilder, VectorBuilder};
pub use errors::{Error, Result};
pub use primitives::{Endian, PrimitiveType};
pub use reader::{MessageReader, VectorReader};
pub use vector::Vector;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        builder::{MessageBuilder, VectorBuilder},
        errors::{Error, Result},
        reader::{MessageReader, VectorReader},
        primitives::{Endian, PrimitiveType},
        vector::Vector,
    };
}

/// Core constants for ZeroProto format
pub mod constants {
    use crate::primitives::Endian;
    
    /// Maximum number of fields in a message
    pub const MAX_FIELDS: u16 = u16::MAX;
    
    /// Field table entry size in bytes
    pub const FIELD_ENTRY_SIZE: usize = 5; // type_id (1) + offset (4)
    
    /// Endianness used by ZeroProto (little-endian)
    pub const ENDIANNESS: Endian = Endian::Little;
}

pub use constants::ENDIANNESS;

/// Trait for types that can be read from ZeroProto buffers
pub trait ZpRead<'a>: Sized {
    /// Read this type from the given buffer at the given offset
    fn read(buf: &'a [u8], offset: usize) -> Result<Self>;
    
    /// Get the size of this type in bytes
    fn size() -> usize;
}

/// Trait for types that can be written to ZeroProto buffers
pub trait ZpWrite: Sized {
    /// Write this type to the given buffer at the given offset
    fn write(&self, buf: &mut [u8], offset: usize) -> Result<()>;
    
    /// Get the size of this type in bytes
    fn size(&self) -> usize;
}

/// Implement ZpRead for primitive types
macro_rules! impl_primitive_read {
    ($ty:ty, $size:expr, $read_method:ident) => {
        impl<'a> ZpRead<'a> for $ty {
            fn read(buf: &'a [u8], offset: usize) -> Result<Self> {
                if offset + $size > buf.len() {
                    return Err(Error::OutOfBounds);
                }
                Ok(ENDIANNESS.$read_method(buf, offset))
            }
            
            fn size() -> usize { $size }
        }
    };
}

/// Implement ZpWrite for primitive types
macro_rules! impl_primitive_write {
    ($ty:ty, $size:expr, $write_method:ident) => {
        impl ZpWrite for $ty {
            fn write(&self, buf: &mut [u8], offset: usize) -> Result<()> {
                if offset + $size > buf.len() {
                    return Err(Error::OutOfBounds);
                }
                ENDIANNESS.$write_method(*self, buf, offset);
                Ok(())
            }
            
            fn size(&self) -> usize { $size }
        }
    };
}

// Implement for all primitive types
impl_primitive_read!(u8, 1, read_u8);
impl_primitive_read!(u16, 2, read_u16);
impl_primitive_read!(u32, 4, read_u32);
impl_primitive_read!(u64, 8, read_u64);
impl_primitive_read!(i8, 1, read_i8);
impl_primitive_read!(i16, 2, read_i16);
impl_primitive_read!(i32, 4, read_i32);
impl_primitive_read!(i64, 8, read_i64);
impl_primitive_read!(f32, 4, read_f32);
impl_primitive_read!(f64, 8, read_f64);
impl_primitive_read!(bool, 1, read_bool);

impl_primitive_write!(u8, 1, write_u8);
impl_primitive_write!(u16, 2, write_u16);
impl_primitive_write!(u32, 4, write_u32);
impl_primitive_write!(u64, 8, write_u64);
impl_primitive_write!(i8, 1, write_i8);
impl_primitive_write!(i16, 2, write_i16);
impl_primitive_write!(i32, 4, write_i32);
impl_primitive_write!(i64, 8, write_i64);
impl_primitive_write!(f32, 4, write_f32);
impl_primitive_write!(f64, 8, write_f64);
impl_primitive_write!(bool, 1, write_bool);
