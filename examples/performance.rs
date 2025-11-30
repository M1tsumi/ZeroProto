//! Performance benchmark example for ZeroProto
//! Demonstrates zero-copy benefits and performance characteristics

use std::time::Instant;
use zeroproto::{MessageBuilder, MessageReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ZeroProto Performance Benchmarks");
    println!("=================================");
    
    // Test different message sizes
    let sizes = vec![10, 100, 1000, 10000];
    
    for &size in &sizes {
        println!("\n📊 Benchmarking message size: {} fields", size);
        
        // Create test data
        let test_data: Vec<u64> = (0..size).map(|i| i * 12345).collect();
        
        // Benchmark serialization
        let start = Instant::now();
        let mut builder = MessageBuilder::new();
        
        for (i, &value) in test_data.iter().enumerate() {
            builder.set_scalar(i as u16, value)?;
        }
        
        let serialized = builder.finish();
        let serialize_time = start.elapsed();
        
        // Benchmark deserialization (zero-copy)
        let start = Instant::now();
        let reader = MessageReader::new(&serialized)?;
        let deserialize_time = start.elapsed();
        
        // Benchmark field access
        let start = Instant::now();
        let mut sum = 0u64;
        for i in 0..size {
            let value: u64 = reader.get_scalar(i as u16)?;
            sum += value;
        }
        let access_time = start.elapsed();
        
        // Verify correctness
        let expected_sum: u64 = test_data.iter().sum();
        assert_eq!(sum, expected_sum, "Sum mismatch!");
        
        // Print results
        println!("   Serialization:   {:>8.2} μs ({:>6.0} ns/field)", 
                 serialize_time.as_micros(), 
                 serialize_time.as_nanos() as f64 / size as f64);
        println!("   Deserialization: {:>8.2} μs ({:>6.0} ns/field)", 
                 deserialize_time.as_micros(),
                 deserialize_time.as_nanos() as f64 / size as f64);
        println!("   Field access:    {:>8.2} μs ({:>6.0} ns/field)", 
                 access_time.as_micros(),
                 access_time.as_nanos() as f64 / size as f64);
        println!("   Total size:       {} bytes", serialized.len());
        println!("   Bytes per field:  {:.1}", serialized.len() as f64 / size as f64);
        
        // Demonstrate zero-copy
        benchmark_zero_copy(&serialized, size)?;
    }
    
    // Benchmark string handling
    println!("\n🧵 String Handling Benchmark");
    benchmark_strings()?;
    
    // Benchmark vectors
    println!("\n📚 Vector Handling Benchmark");
    benchmark_vectors()?;
    
    println!("\n🎯 Performance benchmarks completed!");
    
    Ok(())
}

fn benchmark_zero_copy(data: &[u8], field_count: usize) -> Result<(), Box<dyn std::error::Error>> {
    // Test multiple readers on the same data (zero-copy benefit)
    let iterations = 1000;
    
    let start = Instant::now();
    for _ in 0..iterations {
        let reader = MessageReader::new(data)?;
        // Access first field
        let _: u64 = reader.get_scalar(0)?;
    }
    let time = start.elapsed();
    
    println!("   Zero-copy readers: {:>8.2} μs ({:>6.0} ns/reader)", 
             time.as_micros(),
             time.as_nanos() as f64 / iterations as f64);
    
    Ok(())
}

fn benchmark_strings() -> Result<(), Box<dyn std::error::Error>> {
    let test_strings = vec![
        "Hello",
        "Hello, World!",
        "This is a longer string with more content",
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    ];
    
    for (i, test_string) in test_strings.iter().enumerate() {
        let mut builder = MessageBuilder::new();
        builder.set_string(0, test_string)?;
        let data = builder.finish();
        
        // Benchmark string access
        let iterations = 10000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let reader = MessageReader::new(&data)?;
            let string = reader.get_string(0)?;
            // Verify content (this would be the actual usage)
            assert_eq!(string, *test_string);
        }
        
        let time = start.elapsed();
        println!("   String {} ({} chars): {:>8.2} μs ({:>6.0} ns/access)", 
                 i + 1, 
                 test_string.len(),
                 time.as_micros(),
                 time.as_nanos() as f64 / iterations as f64);
        
        // Verify zero-copy
        let reader = MessageReader::new(&data)?;
        let string = reader.get_string(0)?;
        let string_in_buffer = (string.as_ptr() as usize >= data.as_ptr() as usize) &&
                               (string.as_ptr() as usize < data.as_ptr() as usize + data.len());
        println!("     Zero-copy: {}", if string_in_buffer { "✅" } else { "❌" });
    }
    
    Ok(())
}

fn benchmark_vectors() -> Result<(), Box<dyn std::error::Error>> {
    let vector_sizes = vec![10, 100, 1000];
    
    for &size in &vector_sizes {
        let test_data: Vec<u32> = (0..size).map(|i| i as u32 * 2).collect();
        
        let mut builder = MessageBuilder::new();
        builder.set_vector(0, &test_data)?;
        let data = builder.finish();
        
        // Benchmark vector iteration
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let reader = MessageReader::new(&data)?;
            let vector_reader = reader.get_vector::<u32>(0)?;
            
            // Collect all items (simulating real usage)
            let collected: Vec<u32> = vector_reader.collect()?;
            assert_eq!(collected.len(), size);
        }
        
        let time = start.elapsed();
        println!("   Vector {} items: {:>8.2} μs ({:>6.0} ns/iteration)", 
                 size,
                 time.as_micros(),
                 time.as_nanos() as f64 / iterations as f64);
        
        // Benchmark individual access
        let start = Instant::now();
        for _ in 0..iterations {
            let reader = MessageReader::new(&data)?;
            let vector_reader = reader.get_vector::<u32>(0)?;
            
            // Access first few elements
            for i in 0..std::cmp::min(5, size) {
                let _: u32 = vector_reader.get(i)?;
            }
        }
        
        let time = start.elapsed();
        println!("     Random access: {:>8.2} μs ({:>6.0} ns/access)", 
                 time.as_micros(),
                 time.as_nanos() as f64 / (iterations * std::cmp::min(5, size)) as f64);
    }
    
    Ok(())
}
