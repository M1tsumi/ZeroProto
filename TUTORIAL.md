# ZeroProto Tutorial

This tutorial will walk you through using ZeroProto, from basic concepts to advanced usage patterns.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Basic Concepts](#basic-concepts)
3. [Schema Definition](#schema-definition)
4. [Code Generation](#code-generation)
5. [Using Generated Code](#using-generated-code)
6. [Advanced Features](#advanced-features)
7. [Performance Optimization](#performance-optimization)
8. [Error Handling](#error-handling)
9. [Best Practices](#best-practices)

## Getting Started

### Installation

Add ZeroProto to your `Cargo.toml`:

```toml
[dependencies]
zeroproto = "0.2.0"

[build-dependencies]
zeroproto-compiler = "0.2.0"
```

### Your First Project

Create a new project with the CLI:

```bash
zeroproto init my-project
cd my-project
cargo build
```

This creates a complete project structure with:
- `Cargo.toml` with dependencies
- `build.rs` for schema compilation
- `schemas/` directory for `.zp` files
- `src/main.rs` with example code

## Basic Concepts

### Zero-Copy Serialization

ZeroProto's key innovation is zero-copy deserialization. When you deserialize data:

```rust
// Traditional approach (allocates new memory)
let string: String = deserialize_json(&json_data);

// ZeroProto approach (no allocation)
let string: &str = reader.get_string(0)?;
```

The string slice directly references the original buffer - no copying, no allocation!

### Schema-Based Code Generation

ZeroProto uses schema files (`.zp`) to define your data structures:

```zp
message User {
    id: u64;
    name: string;
    email: string;
}
```

The compiler generates type-safe Rust code:

```rust
// Generated code
pub struct UserReader<'a> {
    reader: MessageReader<'a>,
}

impl<'a> UserReader<'a> {
    pub fn id(&self) -> zeroproto::Result<u64> { /* ... */ }
    pub fn name(&self) -> zeroproto::Result<&'a str> { /* ... */ }
    pub fn email(&self) -> zeroproto::Result<&'a str> { /* ... */ }
}
```

## Schema Definition

### Primitive Types

ZeroProto supports all primitive types:

```zp
message Primitives {
    // Integers
    int8_field: i8;
    int16_field: i16;
    int32_field: i32;
    int64_field: i64;
    uint8_field: u8;
    uint16_field: u16;
    uint32_field: u32;
    uint64_field: u64;
    
    // Floats
    float32_field: f32;
    float64_field: f64;
    
    // Other types
    bool_field: bool;
    string_field: string;
    bytes_field: bytes;
}
```

### Enums

Define enums with explicit values:

```zp
enum Status {
    Inactive = 0;
    Active = 1;
    Pending = 2;
    Suspended = 3;
}

message User {
    id: u64;
    status: Status;
}
```

### Nested Messages

Messages can contain other messages:

```zp
message Address {
    street: string;
    city: string;
    country: string;
    postal_code: string;
}

message User {
    id: u64;
    name: string;
    address: Address;
}
```

### Vectors

Collections of any type:

```zp
message User {
    id: u64;
    tags: [string];
    scores: [u32];
    friends: [u64];
}
```

## Code Generation

### Build Script

Create a `build.rs` file:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile all .zp files in schemas directory
    zeroproto_compiler::build()?;
    Ok(())
}
```

Or compile specific files:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    zeroproto_compiler::compile_multiple(
        &["schemas/user.zp", "schemas/post.zp"],
        "src/generated"
    )?;
    Ok(())
}
```

### Directory Structure

```
my-project/
├── Cargo.toml
├── build.rs
├── schemas/
│   ├── user.zp
│   └── post.zp
└── src/
    ├── main.rs
    └── generated/
        ├── user.rs
        └── post.rs
```

## Using Generated Code

### Building Messages

Use the generated builder to create messages:

```rust
use generated::user::*;

fn create_user() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut builder = UserBuilder::new();
    builder.set_id(12345)?;
    builder.set_name("Alice")?;
    builder.set_email("alice@example.com")?;
    
    // Set nested address
    let mut address_builder = AddressBuilder::new();
    address_builder.set_street("123 Main St")?;
    address_builder.set_city("Anytown")?;
    address_builder.set_country("USA")?;
    address_builder.set_postal_code("12345")?;
    
    let address_data = address_builder.finish();
    builder.set_address(&address_data)?;
    
    // Set tags
    let tags = vec!["developer", "rust", "zeroproto"];
    builder.set_tags(&tags)?;
    
    Ok(builder.finish())
}
```

### Reading Messages

Use the generated reader for zero-copy access:

```rust
fn read_user(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let user = UserReader::from_slice(data)?;
    
    println!("User ID: {}", user.id()?);
    println!("Name: {}", user.name()?);
    println!("Email: {}", user.email()?);
    
    // Read nested address
    let address = user.address()?;
    println!("Address: {}, {}, {}", 
             address.street()?,
             address.city()?,
             address.country()?);
    
    // Iterate over tags
    let tags = user.tags()?;
    for tag in tags.iter() {
        println!("Tag: {}", tag?);
    }
    
    Ok(())
}
```

### Error Handling

All operations return `Result<T, zeroproto::Error>`:

```rust
use zeroproto::Error;

fn handle_user(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    match UserReader::from_slice(data) {
        Ok(user) => {
            match user.id() {
                Ok(id) => println!("User ID: {}", id),
                Err(Error::InvalidFieldType) => println!("Invalid field type"),
                Err(e) => return Err(Box::new(e)),
            }
        }
        Err(e) => return Err(Box::new(e)),
    }
    Ok(())
}
```

## Advanced Features

### Field Offsets

Generated code includes offset constants for direct access:

```rust
// Generated constants
pub const FIELD_0_OFFSET: usize = 27;
pub const FIELD_1_OFFSET: usize = 35;

// Use for manual parsing if needed
let id_offset = FIELD_0_OFFSET;
```

### Custom Validation

Add validation when reading:

```rust
fn validate_user(user: &UserReader) -> Result<(), Box<dyn std::error::Error>> {
    let id = user.id()?;
    if id == 0 {
        return Err("User ID cannot be zero".into());
    }
    
    let email = user.email()?;
    if !email.contains('@') {
        return Err("Invalid email format".into());
    }
    
    Ok(())
}
```

### Partial Updates

Update specific fields without rebuilding the entire message:

```rust
fn update_user_email(mut builder: UserBuilder, new_email: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    builder.set_email(new_email)?;
    Ok(builder.finish())
}
```

## Performance Optimization

### Batch Operations

Process multiple messages efficiently:

```rust
fn process_users(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // Assume data contains multiple concatenated messages
    let mut offset = 0;
    
    while offset < data.len() {
        let reader = MessageReader::new(&data[offset..])?;
        let user = UserReader::new(reader)?;
        
        // Process user
        println!("Processing user: {}", user.name()?);
        
        // Move to next message (you'd need to store message sizes)
        offset += user.reader.total_size();
    }
    
    Ok(())
}
```

### Memory Pooling

Reuse builders for better performance:

```rust
struct UserPool {
    builders: Vec<UserBuilder>,
}

impl UserPool {
    fn new() -> Self {
        Self {
            builders: Vec::with_capacity(100),
        }
    }
    
    fn get_builder(&mut self) -> UserBuilder {
        self.builders.pop().unwrap_or_else(|| UserBuilder::new())
    }
    
    fn return_builder(&mut self, builder: UserBuilder) {
        self.builders.push(builder);
    }
}
```

### Zero-Copy Iteration

Iterate over vectors without allocation:

```rust
fn process_tags(user: &UserReader) -> Result<(), Box<dyn std::error::Error>> {
    let tags = user.tags()?;
    
    // Zero-copy iteration
    for tag in tags.iter() {
        let tag_str = tag?;
        if tag_str.starts_with("rust") {
            println!("Rust-related tag: {}", tag_str);
        }
    }
    
    Ok(())
}
```

## Error Handling

### Error Types

ZeroProto defines several error types:

```rust
use zeroproto::Error;

match error {
    Error::InvalidFieldType => /* Wrong type for field */,
    Error::InvalidOffset => /* Field offset out of bounds */,
    Error::InvalidLength => /* Invalid vector/string length */,
    Error::Io(e) => /* I/O error */,
}
```

### Validation Errors

Schema validation produces detailed errors:

```rust
// Schema validation error
Error::Validation("Field name 'id' is reserved in message 'User'")
```

### Runtime Errors

Runtime errors include context:

```rust
use zeroproto::Error;

fn safe_read(reader: &UserReader) -> Result<String, Error> {
    let id = reader.id()
        .map_err(|e| Error::InvalidFieldType)?;
    
    let name = reader.name()
        .map_err(|e| Error::InvalidFieldType)?;
    
    Ok(format!("{}: {}", id, name))
}
```

## Best Practices

### Schema Design

1. **Use explicit enum values** - Always assign values to enum variants
2. **Avoid reserved names** - Don't use "id", "type", "data", "buffer" for field names
3. **Keep messages small** - Large messages impact performance
4. **Use vectors for collections** - More efficient than repeated fields

### Performance Tips

1. **Reuse builders** - Pool builders for frequent allocations
2. **Batch operations** - Process multiple messages together
3. **Minimize string copies** - Use zero-copy string access
4. **Prefer vectors over repeated fields** - More memory efficient

### Error Handling

1. **Handle all Result types** - Don't unwrap() in production code
2. **Provide context** - Add meaningful error messages
3. **Validate early** - Check schema at compile time
4. **Log errors** - Include context for debugging

### Testing

1. **Test roundtrip** - Serialize then deserialize
2. **Test edge cases** - Empty vectors, large strings
3. **Test errors** - Verify error handling works
4. **Benchmark** - Measure performance in your use case

## Complete Example

Here's a complete example putting everything together:

```rust
// schemas/user.zp
enum Status {
    Inactive = 0;
    Active = 1;
    Pending = 2;
}

message User {
    user_id: u64;
    username: string;
    email: string;
    status: Status;
    tags: [string];
}
```

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    zeroproto_compiler::build()?;
    Ok(())
}
```

```rust
// src/main.rs
mod generated;

use generated::user::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create user
    let user_data = create_test_user()?;
    
    // Read user (zero-copy)
    let user = UserReader::from_slice(&user_data)?;
    
    // Display user info
    display_user(&user)?;
    
    // Validate user
    validate_user(&user)?;
    
    Ok(())
}

fn create_test_user() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut builder = UserBuilder::new();
    builder.set_user_id(12345)?;
    builder.set_username("alice")?;
    builder.set_email("alice@example.com")?;
    builder.set_status(Status::Active)?;
    
    let tags = vec!["developer", "rust", "zeroproto"];
    builder.set_tags(&tags)?;
    
    Ok(builder.finish())
}

fn display_user(user: &UserReader) -> Result<(), Box<dyn std::error::Error>> {
    println!("User Information:");
    println!("  ID: {}", user.user_id()?);
    println!("  Username: {}", user.username()?);
    println!("  Email: {}", user.email()?);
    println!("  Status: {:?}", user.status()?);
    
    println!("  Tags:");
    let tags = user.tags()?;
    for tag in tags.iter() {
        println!("    - {}", tag?);
    }
    
    Ok(())
}

fn validate_user(user: &UserReader) -> Result<(), Box<dyn std::error::Error>> {
    let id = user.user_id()?;
    if id == 0 {
        return Err("User ID cannot be zero".into());
    }
    
    let email = user.email()?;
    if !email.contains('@') {
        return Err("Invalid email format".into());
    }
    
    let tags = user.tags()?;
    if tags.len()? == 0 {
        return Err("User must have at least one tag".into());
    }
    
    println!("✅ User validation passed");
    Ok(())
}
```

This tutorial covers all the essential concepts and patterns for using ZeroProto effectively. The zero-copy design provides significant performance benefits while maintaining type safety and ergonomics.
