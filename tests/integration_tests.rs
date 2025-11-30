//! Integration tests for ZeroProto

use zeroproto::{MessageBuilder, MessageReader};

#[test]
fn test_basic_message_roundtrip() {
    // Create a simple message with scalar fields
    let mut builder = MessageBuilder::new();
    builder.set_scalar(0, 42u32).unwrap();
    builder.set_scalar(1, 3.14f64).unwrap();
    builder.set_scalar(2, true).unwrap();
    
    let data = builder.finish();
    
    // Read it back
    let reader = MessageReader::new(&data).unwrap();
    assert_eq!(reader.field_count(), 3);
    
    let value: u32 = reader.get_scalar(0).unwrap();
    assert_eq!(value, 42);
    
    let value: f64 = reader.get_scalar(1).unwrap();
    assert_eq!(value, 3.14);
    
    let value: bool = reader.get_scalar(2).unwrap();
    assert_eq!(value, true);
}

#[test]
fn test_string_field() {
    let mut builder = MessageBuilder::new();
    builder.set_string(0, "Hello, ZeroProto!").unwrap();
    
    let data = builder.finish();
    
    let reader = MessageReader::new(&data).unwrap();
    let value = reader.get_string(0).unwrap();
    assert_eq!(value, "Hello, ZeroProto!");
}

#[test]
fn test_bytes_field() {
    let bytes = b"binary data";
    let mut builder = MessageBuilder::new();
    builder.set_bytes(0, bytes).unwrap();
    
    let data = builder.finish();
    
    let reader = MessageReader::new(&data).unwrap();
    let value = reader.get_bytes(0).unwrap();
    assert_eq!(value, bytes);
}

#[test]
fn test_vector_field() {
    let values = vec![1u32, 2, 3, 4, 5];
    let mut builder = MessageBuilder::new();
    builder.set_vector(0, &values).unwrap();
    
    let data = builder.finish();
    
    let reader = MessageReader::new(&data).unwrap();
    let vector_reader = reader.get_vector::<u32>(0).unwrap();
    
    assert_eq!(vector_reader.len(), 5);
    let collected = vector_reader.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(collected, values);
}

#[test]
fn test_nested_message() {
    // Create inner message
    let mut inner_builder = MessageBuilder::new();
    inner_builder.set_scalar(0, 123u64).unwrap();
    inner_builder.set_string(1, "inner").unwrap();
    let inner_data = inner_builder.finish();
    
    // Create outer message with nested message
    let mut outer_builder = MessageBuilder::new();
    outer_builder.set_scalar(0, 456u64).unwrap();
    outer_builder.set_message(1, &inner_data).unwrap();
    let outer_data = outer_builder.finish();
    
    // Read and verify
    let outer_reader = MessageReader::new(&outer_data).unwrap();
    let outer_id: u64 = outer_reader.get_scalar(0).unwrap();
    assert_eq!(outer_id, 456);
    
    let inner_reader = outer_reader.get_message(1).unwrap();
    let inner_id: u64 = inner_reader.get_scalar(0).unwrap();
    assert_eq!(inner_id, 123);
    
    let inner_string = inner_reader.get_string(1).unwrap();
    assert_eq!(inner_string, "inner");
}

#[test]
fn test_empty_message() {
    let builder = MessageBuilder::new();
    let data = builder.finish();
    
    let reader = MessageReader::new(&data).unwrap();
    assert_eq!(reader.field_count(), 0);
}

#[test]
fn test_large_message() {
    let mut builder = MessageBuilder::new();
    
    // Add many fields
    for i in 0..100 {
        builder.set_scalar(i, i as u32).unwrap();
    }
    
    let data = builder.finish();
    
    let reader = MessageReader::new(&data).unwrap();
    assert_eq!(reader.field_count(), 100);
    
    for i in 0..100 {
        let value: u32 = reader.get_scalar(i).unwrap();
        assert_eq!(value, i as u32);
    }
}

#[test]
fn test_invalid_message() {
    // Test with too short buffer
    let buffer = vec![0]; // Too short for field count
    assert!(MessageReader::new(&buffer).is_err());
    
    // Test with invalid field count
    let buffer = vec![255, 255]; // u16::MAX fields
    assert!(MessageReader::new(&buffer).is_err());
}

#[test]
fn test_out_of_bounds_access() {
    let mut builder = MessageBuilder::new();
    builder.set_scalar(0, 42u32).unwrap();
    let data = builder.finish();
    
    let reader = MessageReader::new(&data).unwrap();
    
    // Try to access non-existent field
    let result: Result<u32, _> = reader.get_scalar(1);
    assert!(result.is_err());
}

#[test]
fn test_string_utf8_validation() {
    let mut builder = MessageBuilder::new();
    builder.set_string(0, "valid utf-8").unwrap();
    let data = builder.finish();
    
    let reader = MessageReader::new(&data).unwrap();
    let value = reader.get_string(0).unwrap();
    assert_eq!(value, "valid utf-8");
    
    // Note: We can't test invalid UTF-8 directly through the builder
    // since it only accepts valid strings
}
