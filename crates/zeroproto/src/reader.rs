//! Message and vector readers for ZeroProto deserialization

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::{
    constants::FIELD_ENTRY_SIZE,
    errors::{Error, Result},
    primitives::{Endian, PrimitiveType},
    ZpRead,
};

/// A zero-copy message reader
#[derive(Debug)]
pub struct MessageReader<'a> {
    buffer: &'a [u8],
    field_count: u16,
    field_table_offset: usize,
}

impl<'a> MessageReader<'a> {
    /// Create a new message reader from a buffer
    pub fn new(buffer: &'a [u8]) -> Result<Self> {
        if buffer.len() < 2 {
            return Err(Error::InvalidMessage);
        }

        let field_count = Endian::Little.read_u16(buffer, 0);

        let field_table_size = field_count as usize * FIELD_ENTRY_SIZE;
        let total_header_size = 2 + field_table_size;

        if buffer.len() < total_header_size {
            return Err(Error::InvalidMessage);
        }

        Ok(Self {
            buffer,
            field_count,
            field_table_offset: 2,
        })
    }

    /// Get the number of fields in this message
    pub fn field_count(&self) -> u16 {
        self.field_count
    }

    /// Get the field table entry for a given field index
    fn field_entry(&self, field_index: u16) -> Result<Option<(PrimitiveType, usize)>> {
        if field_index >= self.field_count {
            return Err(Error::OutOfBounds);
        }

        let entry_offset = self.field_table_offset + field_index as usize * FIELD_ENTRY_SIZE;
        let type_id = self.buffer[entry_offset];
        let primitive_type = PrimitiveType::from_u8(type_id).ok_or(Error::InvalidFieldType)?;

        if primitive_type == PrimitiveType::Unset {
            return Ok(None);
        }

        let offset = Endian::Little.read_u32(self.buffer, entry_offset + 1) as usize;
        Ok(Some((primitive_type, offset)))
    }

    /// Check whether a field entry exists and is set
    pub fn has_field(&self, field_index: u16) -> Result<bool> {
        if field_index >= self.field_count {
            return Ok(false);
        }

        let entry_offset = self.field_table_offset + field_index as usize * FIELD_ENTRY_SIZE;
        let type_id = self.buffer[entry_offset];
        Ok(type_id != PrimitiveType::Unset as u8)
    }

    /// Get a scalar field value
    pub fn try_get_scalar<T: ZpRead<'a>>(&self, field_index: u16) -> Result<Option<T>> {
        match self.field_entry(field_index)? {
            Some((_, field_offset)) => T::read(self.buffer, field_offset).map(Some),
            None => Ok(None),
        }
    }

    /// Get a scalar field value (error if missing)
    pub fn get_scalar<T: ZpRead<'a>>(&self, field_index: u16) -> Result<T> {
        self.try_get_scalar(field_index)?.ok_or(Error::MissingField)
    }

    /// Get a string field
    pub fn get_string(&self, field_index: u16) -> Result<&'a str> {
        self.try_get_string(field_index)?.ok_or(Error::MissingField)
    }

    /// Get a bytes field
    pub fn get_bytes(&self, field_index: u16) -> Result<&'a [u8]> {
        self.try_get_bytes(field_index)?.ok_or(Error::MissingField)
    }

    /// Get a nested message
    pub fn get_message(&self, field_index: u16) -> Result<MessageReader<'a>> {
        self.try_get_message(field_index)?
            .ok_or(Error::MissingField)
    }

    /// Get a vector field
    pub fn get_vector<T: ZpRead<'a>>(&self, field_index: u16) -> Result<VectorReader<'a, T>> {
        self.try_get_vector(field_index)?.ok_or(Error::MissingField)
    }

    /// Try to get a string field
    pub fn try_get_string(&self, field_index: u16) -> Result<Option<&'a str>> {
        match self.field_entry(field_index)? {
            Some((field_type, field_offset)) => {
                if field_type != PrimitiveType::String {
                    return Err(Error::InvalidFieldType);
                }

                let len = Endian::Little.read_u32(self.buffer, field_offset) as usize;
                let string_offset = field_offset + 4;

                if string_offset + len > self.buffer.len() {
                    return Err(Error::OutOfBounds);
                }

                let string_bytes = &self.buffer[string_offset..string_offset + len];
                core::str::from_utf8(string_bytes)
                    .map(Some)
                    .map_err(|_| Error::InvalidUtf8)
            }
            None => Ok(None),
        }
    }

    /// Try to get a bytes field
    pub fn try_get_bytes(&self, field_index: u16) -> Result<Option<&'a [u8]>> {
        match self.field_entry(field_index)? {
            Some((field_type, field_offset)) => {
                if field_type != PrimitiveType::Bytes {
                    return Err(Error::InvalidFieldType);
                }

                let len = Endian::Little.read_u32(self.buffer, field_offset) as usize;
                let bytes_offset = field_offset + 4;

                if bytes_offset + len > self.buffer.len() {
                    return Err(Error::OutOfBounds);
                }

                Ok(Some(&self.buffer[bytes_offset..bytes_offset + len]))
            }
            None => Ok(None),
        }
    }

    /// Try to get a nested message
    pub fn try_get_message(&self, field_index: u16) -> Result<Option<MessageReader<'a>>> {
        match self.field_entry(field_index)? {
            Some((field_type, field_offset)) => {
                if field_type != PrimitiveType::Message {
                    return Err(Error::InvalidFieldType);
                }

                let len = Endian::Little.read_u32(self.buffer, field_offset) as usize;
                let message_offset = field_offset + 4;

                if message_offset + len > self.buffer.len() {
                    return Err(Error::OutOfBounds);
                }

                let message_buffer = &self.buffer[message_offset..message_offset + len];
                Ok(Some(MessageReader::new(message_buffer)?))
            }
            None => Ok(None),
        }
    }

    /// Try to get a vector field
    pub fn try_get_vector<T: ZpRead<'a>>(
        &self,
        field_index: u16,
    ) -> Result<Option<VectorReader<'a, T>>> {
        match self.field_entry(field_index)? {
            Some((field_type, field_offset)) => {
                if field_type != PrimitiveType::Vector {
                    return Err(Error::InvalidFieldType);
                }

                let count = Endian::Little.read_u32(self.buffer, field_offset) as usize;
                let vector_offset = field_offset + 4;

                if vector_offset + count * T::size() > self.buffer.len() {
                    return Err(Error::OutOfBounds);
                }

                Ok(Some(VectorReader {
                    buffer: self.buffer,
                    offset: vector_offset,
                    count,
                    _phantom: core::marker::PhantomData,
                }))
            }
            None => Ok(None),
        }
    }
}

/// A zero-copy vector reader
#[derive(Debug)]
pub struct VectorReader<'a, T> {
    buffer: &'a [u8],
    offset: usize,
    count: usize,
    _phantom: core::marker::PhantomData<T>,
}

impl<'a, T: ZpRead<'a>> VectorReader<'a, T> {
    /// Get the number of elements in the vector
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Get an element at the given index
    pub fn get(&self, index: usize) -> Result<T> {
        if index >= self.count {
            return Err(Error::OutOfBounds);
        }

        let element_offset = self.offset + index * T::size();
        T::read(self.buffer, element_offset)
    }

    /// Get an iterator over the elements
    pub fn iter(&self) -> Box<dyn Iterator<Item = Result<T>> + '_> {
        Box::new((0..self.count).map(move |i| self.get(i)))
    }

    /// Collect all elements into a Vec
    pub fn collect(&self) -> Result<Vec<T>> {
        self.iter().collect()
    }
}

impl<'a, T: ZpRead<'a> + 'a> IntoIterator for VectorReader<'a, T> {
    type Item = Result<T>;
    type IntoIter = Box<dyn Iterator<Item = Result<T>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new((0..self.count).map(move |i| self.get(i)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::MessageBuilder;
    use crate::primitives::Endian;

    #[cfg(feature = "std")]
    use std::println;
    #[cfg(feature = "std")]
    use std::vec;

    #[test]
    fn test_message_reader_creation() {
        let buffer = vec![0, 0]; // Empty message with 0 fields
        let reader = MessageReader::new(&buffer).unwrap();
        assert_eq!(reader.field_count(), 0);
    }

    #[test]
    fn test_invalid_message_too_short() {
        let buffer = vec![0]; // Too short for field count
        assert!(matches!(
            MessageReader::new(&buffer),
            Err(Error::InvalidMessage)
        ));
    }

    #[test]
    fn test_scalar_field() {
        let mut builder = MessageBuilder::new();
        builder.set_scalar(0, 42u16).unwrap();
        let buffer = builder.finish();

        println!("Reader buffer: {:?}", buffer);

        let reader = MessageReader::new(&buffer).unwrap();
        let value: u16 = reader.get_scalar(0).unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_reader_basic() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_scalar(0, 42u64)?;
        let data = builder.finish();

        let reader = MessageReader::new(&data)?;
        let value: u64 = reader.get_scalar(0)?;
        assert_eq!(value, 42);

        Ok(())
    }

    #[test]
    fn test_reader_string() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_string(0, "hello")?;
        let data = builder.finish();

        let reader = MessageReader::new(&data)?;
        let value = reader.get_string(0)?;
        assert_eq!(value, "hello");

        Ok(())
    }

    #[test]
    fn test_reader_bytes() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_bytes(0, b"world")?;
        let data = builder.finish();

        let reader = MessageReader::new(&data)?;
        let value = reader.get_bytes(0)?;
        assert_eq!(value, b"world");

        Ok(())
    }

    #[test]
    fn test_reader_vector() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_vector(0, &[1u32, 2u32, 3u32])?;
        let data = builder.finish();

        let reader = MessageReader::new(&data)?;
        let vector = reader.get_vector::<u32>(0)?;
        let values: Vec<u32> = vector.collect()?;
        assert_eq!(values, vec![1, 2, 3]);

        Ok(())
    }

    #[test]
    fn test_reader_nested_message() -> Result<()> {
        let mut nested_builder = MessageBuilder::new();
        nested_builder.set_scalar(0, 42u64)?;
        let nested_data = nested_builder.finish();

        let mut builder = MessageBuilder::new();
        builder.set_message(0, &nested_data)?;
        let data = builder.finish();

        let reader = MessageReader::new(&data)?;
        let nested_reader = reader.get_message(0)?;
        let value: u64 = nested_reader.get_scalar(0)?;
        assert_eq!(value, 42);

        Ok(())
    }

    #[test]
    fn test_reader_field_count() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_scalar(0, 1u8)?;
        builder.set_scalar(1, 2u8)?;
        builder.set_scalar(2, 3u8)?;
        let data = builder.finish();

        let reader = MessageReader::new(&data)?;
        assert_eq!(reader.field_count(), 3);

        Ok(())
    }

    #[test]
    fn test_reader_invalid_field() {
        let mut builder = MessageBuilder::new();
        builder.set_scalar(0, 42u64).unwrap();
        let data = builder.finish();

        let reader = MessageReader::new(&data).unwrap();
        let result: Result<u64> = reader.get_scalar(1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::OutOfBounds));
    }

    #[test]
    fn test_reader_zero_copy() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_string(0, "hello world")?;
        let data = builder.finish();

        let reader = MessageReader::new(&data)?;
        let string = reader.get_string(0)?;

        // Verify zero-copy
        let string_ptr = string.as_ptr();
        let data_ptr = data.as_ptr();

        assert!(string_ptr as usize >= data_ptr as usize);
        assert!((string_ptr as usize) < data_ptr as usize + data.len());

        Ok(())
    }
}
