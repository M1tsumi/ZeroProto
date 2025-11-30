//! Message and vector builders for ZeroProto serialization

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use crate::{errors::{Error, Result}, primitives::{Endian, PrimitiveType}, ZpWrite, constants::{FIELD_ENTRY_SIZE, MAX_FIELDS}};

/// A message builder for serializing ZeroProto messages
#[derive(Debug)]
pub struct MessageBuilder {
    buffer: Vec<u8>,
    field_entries: Vec<FieldEntry>,
    field_count: u16,
    payload_offset: usize,
}

#[derive(Debug, Clone)]
struct FieldEntry {
    type_id: u8,
    offset: u32,
}

impl MessageBuilder {
    /// Create a new message builder
    pub fn new() -> Self {
        let mut builder = Self {
            buffer: Vec::new(),
            field_entries: Vec::new(),
            field_count: 0,
            payload_offset: 0,
        };
        
        // Reserve space for field count (will be filled later)
        builder.buffer.extend_from_slice(&[0, 0]);
        builder.payload_offset = 2;
        builder
    }

    /// Get the current number of fields
    pub fn field_count(&self) -> u16 {
        self.field_count
    }

    /// Add a scalar field
    pub fn set_scalar<T: ZpWrite>(&mut self, field_index: u16, value: T) -> Result<()> {
        self.ensure_field_index(field_index)?;
        
        let type_id = self.get_type_id::<T>()?;
        let field_offset = self.payload_offset as u32;
        
        // Resize buffer to fit the value
        let required_size = self.payload_offset + value.size();
        if required_size > self.buffer.len() {
            self.buffer.resize(required_size, 0);
        }
        
        // Write the value
        value.write(&mut self.buffer, self.payload_offset)?;
        
        // Add/update field entry
        self.set_field_entry(field_index, type_id, field_offset);
        
        // Update payload offset
        self.payload_offset += value.size();
        
        Ok(())
    }

    /// Add a string field
    pub fn set_string(&mut self, field_index: u16, value: &str) -> Result<()> {
        self.ensure_field_index(field_index)?;
        
        let type_id = PrimitiveType::String as u8;
        let field_offset = self.payload_offset as u32;
        
        // Reserve space for length + string bytes
        let len = value.len();
        let required_size = self.payload_offset + 4 + len;
        if required_size > self.buffer.len() {
            self.buffer.resize(required_size, 0);
        }
        
        // Write length
        Endian::Little.write_u32(len as u32, &mut self.buffer, self.payload_offset);
        
        // Write string bytes
        let string_offset = self.payload_offset + 4;
        self.buffer[string_offset..string_offset + len].copy_from_slice(value.as_bytes());
        
        // Add/update field entry
        self.set_field_entry(field_index, type_id, field_offset);
        
        // Update payload offset
        self.payload_offset += 4 + len;
        
        Ok(())
    }

    /// Add a bytes field
    pub fn set_bytes(&mut self, field_index: u16, value: &[u8]) -> Result<()> {
        self.ensure_field_index(field_index)?;
        
        let type_id = PrimitiveType::Bytes as u8;
        let field_offset = self.payload_offset as u32;
        
        // Reserve space for length + bytes
        let len = value.len();
        let required_size = self.payload_offset + 4 + len;
        if required_size > self.buffer.len() {
            self.buffer.resize(required_size, 0);
        }
        
        // Write length
        Endian::Little.write_u32(len as u32, &mut self.buffer, self.payload_offset);
        
        // Write bytes
        let bytes_offset = self.payload_offset + 4;
        self.buffer[bytes_offset..bytes_offset + len].copy_from_slice(value);
        
        // Add/update field entry
        self.set_field_entry(field_index, type_id, field_offset);
        
        // Update payload offset
        self.payload_offset += 4 + len;
        
        Ok(())
    }

    /// Add a nested message
    pub fn set_message(&mut self, field_index: u16, message: &[u8]) -> Result<()> {
        self.ensure_field_index(field_index)?;
        
        let type_id = PrimitiveType::Message as u8;
        let field_offset = self.payload_offset as u32;
        
        // Reserve space for length + message bytes
        let len = message.len();
        let required_size = self.payload_offset + 4 + len;
        if required_size > self.buffer.len() {
            self.buffer.resize(required_size, 0);
        }
        
        // Write length
        Endian::Little.write_u32(len as u32, &mut self.buffer, self.payload_offset);
        
        // Write message bytes
        let message_offset = self.payload_offset + 4;
        self.buffer[message_offset..message_offset + len].copy_from_slice(message);
        
        // Add/update field entry
        self.set_field_entry(field_index, type_id, field_offset);
        
        // Update payload offset
        self.payload_offset += 4 + len;
        
        Ok(())
    }

    /// Add a vector field
    pub fn set_vector<T: ZpWrite>(&mut self, field_index: u16, values: &[T]) -> Result<()> {
        self.ensure_field_index(field_index)?;
        
        let type_id = PrimitiveType::Vector as u8;
        let field_offset = self.payload_offset as u32;
        
        // Calculate total size needed
        let element_size = if values.is_empty() { 0 } else { values[0].size() };
        let total_size = 4 + values.len() * element_size;
        let required_size = self.payload_offset + total_size;
        if required_size > self.buffer.len() {
            self.buffer.resize(required_size, 0);
        }
        
        // Write count
        Endian::Little.write_u32(values.len() as u32, &mut self.buffer, self.payload_offset);
        
        // Write elements
        let mut offset = self.payload_offset + 4;
        for value in values {
            value.write(&mut self.buffer, offset)?;
            offset += value.size();
        }
        
        // Add/update field entry
        self.set_field_entry(field_index, type_id, field_offset);
        
        // Update payload offset
        self.payload_offset += total_size;
        
        Ok(())
    }

    /// Ensure the field index is valid and resize field entries if needed
    fn ensure_field_index(&mut self, field_index: u16) -> Result<()> {
        if field_index >= MAX_FIELDS {
            return Err(Error::OutOfBounds);
        }
        
        if field_index >= self.field_count {
            self.field_count = field_index + 1;
            self.field_entries.resize(self.field_count as usize, FieldEntry { type_id: 0, offset: 0 });
        }
        
        Ok(())
    }

    /// Set a field entry
    fn set_field_entry(&mut self, field_index: u16, type_id: u8, offset: u32) {
        self.field_entries[field_index as usize] = FieldEntry { type_id, offset };
    }

    /// Get the type ID for a type
    fn get_type_id<T>(&self) -> Result<u8> {
        let type_id = match core::any::type_name::<T>() {
            "u8" => PrimitiveType::U8 as u8,
            "u16" => PrimitiveType::U16 as u8,
            "u32" => PrimitiveType::U32 as u8,
            "u64" => PrimitiveType::U64 as u8,
            "i8" => PrimitiveType::I8 as u8,
            "i16" => PrimitiveType::I16 as u8,
            "i32" => PrimitiveType::I32 as u8,
            "i64" => PrimitiveType::I64 as u8,
            "f32" => PrimitiveType::F32 as u8,
            "f64" => PrimitiveType::F64 as u8,
            "bool" => PrimitiveType::Bool as u8,
            _ => return Err(Error::InvalidFieldType),
        };
        Ok(type_id)
    }

    /// Finish building and return the serialized message
    pub fn finish(mut self) -> Vec<u8> {
        // Write field count
        Endian::Little.write_u16(self.field_count, &mut self.buffer, 0);
        
        // Reserve space for field table
        let field_table_size = self.field_count as usize * FIELD_ENTRY_SIZE;
        let current_payload_offset = self.payload_offset;
        
        // Shift payload to make room for field table
        self.buffer.resize(current_payload_offset + field_table_size, 0);
        
        // Move payload data
        for i in (0..(current_payload_offset - 2)).rev() {
            self.buffer[2 + field_table_size + i] = self.buffer[2 + i];
        }
        
        // Update field entry offsets to account for field table
        for entry in &mut self.field_entries {
            entry.offset += field_table_size as u32;
        }
        
        // Write field table
        let mut field_table_offset = 2;
        for entry in &self.field_entries {
            self.buffer[field_table_offset] = entry.type_id;
            Endian::Little.write_u32(entry.offset, &mut self.buffer, field_table_offset + 1);
            field_table_offset += FIELD_ENTRY_SIZE;
        }
        
        // Trim buffer to actual size
        self.buffer.truncate(2 + field_table_size + (current_payload_offset - 2));
        
        self.buffer
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A vector builder for serializing vectors
#[derive(Debug)]
pub struct VectorBuilder<T> {
    elements: Vec<T>,
}

impl<T: ZpWrite> VectorBuilder<T> {
    /// Create a new vector builder
    pub fn new() -> Self {
        Self { elements: Vec::new() }
    }

    /// Add an element to the vector
    pub fn push(&mut self, element: T) {
        self.elements.push(element);
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the vector is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Finish building and return the elements
    pub fn finish(self) -> Vec<T> {
        self.elements
    }
}

impl<T> Default for VectorBuilder<T> {
    fn default() -> Self {
        Self { elements: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::Endian;
    
    #[cfg(feature = "std")]
    use std::vec;
    #[cfg(feature = "std")]
    use std::println;

    #[test]
    fn test_empty_message() {
        let builder = MessageBuilder::new();
        let buffer = builder.finish();
        assert_eq!(buffer, vec![0, 0]);
    }

    #[test]
    fn test_scalar_field() {
        let mut builder = MessageBuilder::new();
        builder.set_scalar(0, 42u16).unwrap();
        let buffer = builder.finish();
        
        // Parse and verify
        let reader = crate::reader::MessageReader::new(&buffer).unwrap();
        let value: u16 = reader.get_scalar(0).unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_string_field() {
        let mut builder = MessageBuilder::new();
        builder.set_string(0, "hello").unwrap();
        let buffer = builder.finish();
        
        // Parse and verify
        let reader = crate::reader::MessageReader::new(&buffer).unwrap();
        let value = reader.get_string(0).unwrap();
        assert_eq!(value, "hello");
    }

    #[test]
    fn test_builder_basic() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_scalar(0, 42u64)?;
        let data = builder.finish();
        
        assert_eq!(data.len(), 15); // 2 + 5 + 8
        assert_eq!(data[0], 1); // field count
        assert_eq!(data[2], 3); // u64 type id
        
        Ok(())
    }
    
    #[test]
    fn test_builder_multiple_fields() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_scalar(0, 42u64)?;
        builder.set_scalar(1, 100u32)?;
        let data = builder.finish();
        
        println!("Buffer length: {}", data.len());
        println!("Buffer: {:?}", data);
        
        // Expected: 2 (field count) + 10 (field table: 2 fields * 5 bytes) + 8 (u64) + 4 (u32) = 24
        assert_eq!(data.len(), 24);
        
        Ok(())
    }
    
    #[test]
    fn test_builder_string() -> Result<()> {
        let mut builder = MessageBuilder::new();
        builder.set_string(0, "hello")?;
        let data = builder.finish();
        
        println!("String buffer length: {}", data.len());
        println!("String buffer: {:?}", data);
        
        // Expected: 2 (field count) + 5 (field table) + 4 (string length) + 5 (string bytes) = 16
        assert_eq!(data.len(), 16);
        
        Ok(())
    }
}
