# test-project

A ZeroProto project.

## Usage

1. Edit your schema files in the `schemas/` directory
2. Run `cargo build` to generate Rust code
3. Use the generated types in your application

Need to work with a subset of schemas? Every CLI command supports the same filters:

```bash
# Compile only tenant A schemas
zeroproto compile schemas --include "tenants/a/**/*.zp"

# Skip shared legacy files while watching
zeroproto watch schemas --exclude "**/legacy/**"

# Inspect schema stats before changing them
zeroproto inspect schemas --verbose
```

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

    // Optional fields use ergonomic helpers
    builder.set_optional_nickname(Some("ally"));
    builder.clear_nickname();
    
    let data = builder.finish();
    
    // Read the user (zero-copy!)
    let user = UserReader::from_slice(&data)?;
    if let Some(nickname) = user.nickname()? {
        println!("Nickname: {}", nickname);
    }
    println!("User: {}", user.username());
    
    Ok(())
}
```
