# test-project

A ZeroProto project.

## Usage

1. Edit your schema files in the `schemas/` directory
2. Run `cargo build` to generate Rust code
3. Use the generated types in your application

## Generated Code

The Rust code generated from your schemas will be available in `src/generated/`.

## Example

```rust
mod generated;
use generated::example::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a user
    let mut builder = UserBuilder::new();
    builder.set_id(123);
    builder.set_username("alice");
    builder.set_email("alice@example.com");
    builder.set_age(25);
    
    let data = builder.finish();
    
    // Read the user (zero-copy!)
    let user = UserReader::from_slice(&data)?;
    println!("User: {}", user.username());
    
    Ok(())
}
```
