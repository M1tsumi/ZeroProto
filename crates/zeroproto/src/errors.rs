//! Error types for ZeroProto

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

/// Result type for ZeroProto operations
pub type Result<T> = core::result::Result<T, Error>;

/// Errors that can occur during ZeroProto operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Buffer is too small for the requested operation
    OutOfBounds,
    /// Invalid field type for the requested operation
    InvalidFieldType,
    /// Invalid UTF-8 in string field
    InvalidUtf8,
    /// Invalid data format
    InvalidFormat,
    /// Invalid message format
    InvalidMessage,
    /// Requested field is not present in the buffer
    MissingField,
    /// Custom error message
    Custom(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::OutOfBounds => write!(f, "Buffer out of bounds"),
            Error::InvalidFieldType => write!(f, "Invalid field type"),
            Error::InvalidUtf8 => write!(f, "Invalid UTF-8 string"),
            Error::InvalidFormat => write!(f, "Invalid data format"),
            Error::InvalidMessage => write!(f, "Invalid message format"),
            Error::MissingField => write!(f, "Field not present"),
            Error::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl From<core::str::Utf8Error> for Error {
    fn from(_: core::str::Utf8Error) -> Self {
        Error::InvalidUtf8
    }
}

#[cfg(feature = "std")]
impl From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Self {
        Error::InvalidUtf8
    }
}
