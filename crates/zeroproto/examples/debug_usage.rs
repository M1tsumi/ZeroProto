//! Debug example to understand ZeroProto field indexing

use zeroproto::{MessageBuilder, MessageReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ZeroProto Debug Example");
    println!("========================");

    // Create a simple message with just one field
    let mut builder = MessageBuilder::new();
    builder.set_scalar(0, 42u64)?;
    let data = builder.finish();

    println!("Serialized data: {:?}", data);
    println!("Data length: {}", data.len());

    // Manual parsing to understand the format
    if data.len() >= 2 {
        let field_count = u16::from_le_bytes([data[0], data[1]]);
        println!("Field count from header: {}", field_count);

        if data.len() >= 7 {
            // 2 + 5 bytes for field entry
            let type_id = data[2];
            let offset = u32::from_le_bytes([data[3], data[4], data[5], data[6]]);
            println!("Field 0 - Type: {}, Offset: {}", type_id, offset);

            if data.len() > offset as usize + 8 {
                let value_bytes = &data[offset as usize..offset as usize + 8];
                let value = u64::from_le_bytes(value_bytes.try_into().unwrap());
                println!("Value bytes: {:?}", value_bytes);
                println!("Expected value: {}", value);
            }
        }
    }

    // Try to read it back
    let reader = MessageReader::new(&data)?;
    println!("Reader field count: {}", reader.field_count());

    // Try to read field 0
    match reader.get_scalar::<u64>(0) {
        Ok(value) => println!("Reader field 0: {}", value),
        Err(e) => println!("Error reading field 0: {:?}", e),
    }

    Ok(())
}
