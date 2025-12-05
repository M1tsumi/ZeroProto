//! Basic usage example for ZeroProto

use zeroproto::{MessageBuilder, MessageReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ZeroProto Basic Usage Example");
    println!("=============================");

    // Create a message with sequential field indices
    let mut builder = MessageBuilder::new();

    // Add scalar fields (indices 0, 1, 2)
    builder.set_scalar(0, 12345u64)?;
    builder.set_scalar(1, 3.14159f64)?;
    builder.set_scalar(2, true)?;

    // Add a string field (index 3)
    builder.set_string(3, "Hello, ZeroProto!")?;

    // Add a bytes field (index 4)
    let binary_data = b"binary\x00data\x01";
    builder.set_bytes(4, binary_data)?;

    // Add a vector field (index 5)
    let numbers = vec![1u32, 2u32, 3u32, 4u32, 5u32];
    builder.set_vector(5, &numbers)?;

    // Add a nested message (index 6)
    let mut nested_builder = MessageBuilder::new();
    nested_builder.set_scalar(0, 999u32)?;
    nested_builder.set_string(1, "nested message")?;
    let nested_data = nested_builder.finish();

    builder.set_message(6, &nested_data)?;

    // Finish building and get the serialized data
    let serialized_data = builder.finish();

    println!("✅ Serialized message: {} bytes", serialized_data.len());
    println!(
        "   First few bytes: {:?}",
        &serialized_data[..std::cmp::min(16, serialized_data.len())]
    );

    // Deserialize the message (zero-copy!)
    let reader = MessageReader::new(&serialized_data)?;

    println!("\n📖 Reading message (zero-copy):");
    println!("   Field count: {}", reader.field_count());

    // Read scalar fields using matching indices
    let id: u64 = reader.get_scalar(0)?;
    let pi: f64 = reader.get_scalar(1)?;
    let flag: bool = reader.get_scalar(2)?;

    println!("   ID: {}", id);
    println!("   PI: {:.5}", pi);
    println!("   Flag: {}", flag);

    // Read string field
    let message = reader.get_string(3)?;
    println!("   Message: {}", message);

    // Read bytes field
    let data = reader.get_bytes(4)?;
    println!("   Binary data: {:?} ({} bytes)", data, data.len());

    // Read vector field
    let vector_reader = reader.get_vector::<u32>(5)?;
    let vector_data: Vec<u32> = vector_reader.collect()?;
    println!("   Vector: {:?}", vector_data);

    // Read nested message
    let nested_reader = reader.get_message(6)?;
    let nested_id: u32 = nested_reader.get_scalar(0)?;
    let nested_message = nested_reader.get_string(1)?;

    println!("   Nested message:");
    println!("     ID: {}", nested_id);
    println!("     Message: {}", nested_message);

    println!("\n🎉 Success! ZeroProto is working correctly.");

    // Demonstrate zero-copy by showing that the string slice
    // references the original buffer without allocation
    let string_slice = reader.get_string(3)?;
    let string_ptr = string_slice.as_ptr();
    let buffer_ptr = serialized_data.as_ptr();

    println!("\n🔍 Zero-copy verification:");
    println!("   String slice pointer: {:p}", string_ptr);
    println!("   Buffer pointer:      {:p}", buffer_ptr);
    println!(
        "   String lives in buffer: {}",
        (string_ptr as usize) >= (buffer_ptr as usize)
            && (string_ptr as usize) < (buffer_ptr as usize + serialized_data.len())
    );

    Ok(())
}
