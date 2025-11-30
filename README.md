# ZeroProto

[![Crates.io](https://img.shields.io/crates/v/zeroproto.svg)](https://crates.io/crates/zeroproto)
[![Documentation](https://docs.rs/zeroproto/badge.svg)](https://docs.rs/zeroproto)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-APACHE)
[![Build Status](https://github.com/zeroproto/zeroproto/workflows/CI/badge.svg)](https://github.com/zeroproto/zeroproto/actions)
[![Coverage](https://codecov.io/gh/zeroproto/zeroproto/branch/main/graph/badge.svg)](https://codecov.io/gh/zeroproto/zeroproto)

ZeroProto is a **zero-copy binary serialization format** designed for high-performance Rust applications. It provides schema-based code generation with compile-time type safety and runtime performance that rivals hand-optimized protocols.

## Features

- **Zero-Copy Deserialization** - Read data directly from buffers without allocation
- **Schema-Based Code Generation** - Define messages in `.zp` schema files
- **Type-Safe API** - Generated Rust code with compile-time guarantees
- **High Performance** - Optimized for speed with minimal overhead
- **Memory Safe** - No unsafe code in public APIs
- **no_std Support** - Works in embedded environments
- **Cross-Platform** - Little-endian format for consistency
- **Rich Type System** - Primitives, strings, bytes, vectors, and nested messages

## Quick Start

### Installation

Add ZeroProto to your `Cargo.toml`:

```toml
[dependencies]
zeroproto = "0.1.0"

[build-dependencies]
zeroproto-compiler = "0.1.0"
```

### Define a Schema

Create a `schemas/user.zp` file:

```zp
message User {
    id: u64;
    username: string;
    email: string;
    age: u8;
    friends: [u64];
    profile: Profile;
}

message Profile {
    bio: string;
    avatar_url: string;
    settings: UserSettings;
}

message UserSettings {
    theme: Theme;
    notifications_enabled: bool;
    max_friends: u32;
}

enum Theme {
    Light = 0;
    Dark = 1;
    Auto = 2;
}
```

### Generate Code

Create a `build.rs` file:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    zeroproto_compiler::build()?;
    Ok(())
}
```

### Use Generated Types

```rust
mod generated;
use generated::user::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a user
    let mut builder = UserBuilder::new();
    builder.set_id(12345);
    builder.set_username("alice");
    builder.set_email("alice@example.com");
    builder.set_age(25);
    
    let mut friends = vec![1001, 1002, 1003];
    builder.set_friends(&friends);
    
    let mut profile_builder = ProfileBuilder::new();
    profile_builder.set_bio("Software developer");
    profile_builder.set_avatar_url("https://example.com/avatar.jpg");
    
    let mut settings_builder = UserSettingsBuilder::new();
    settings_builder.set_theme(Theme::Dark);
    settings_builder.set_notifications_enabled(true);
    settings_builder.set_max_friends(500);
    
    let settings_data = settings_builder.finish();
    profile_builder.set_settings(&settings_data);
    
    let profile_data = profile_builder.finish();
    builder.set_profile(&profile_data);
    
    let user_data = builder.finish();
    
    // Read the user (zero-copy!)
    let user = UserReader::from_slice(&user_data)?;
    println!("User: {}", user.username());
    println!("Email: {}", user.email());
    println!("Age: {}", user.age());
    
    // Access nested data
    let profile = user.profile()?;
    println!("Bio: {}", profile.bio());
    
    let settings = profile.settings()?;
    println!("Theme: {:?}", settings.theme());
    println!("Notifications: {}", settings.notifications_enabled());
    
    // Iterate over friends
    let friends_reader = user.friends()?;
    for friend_id in friends_reader.iter() {
        println!("Friend ID: {}", friend_id?);
    }
    
    Ok(())
}
```

## Documentation

- [API Documentation](https://docs.rs/zeroproto)
- [Binary Format Specification](docs/specifications.md)
- [Schema Language Guide](docs/schema-guide.md)
- [Performance Benchmarks](docs/benchmarks.md)
- [Migration Guide](docs/migration.md)

## Architecture

ZeroProto consists of several crates:

- **`zeroproto`** - Core runtime library with readers and builders
- **`zeroproto-compiler`** - Schema compiler and code generator with hand-written recursive descent parser
- **`zeroproto-macros`** - Procedural macros for derive support
- **`zeroproto-cli`** - Command-line interface for development

### Compiler Pipeline

The compilation process follows these steps:

1. **Parsing** - Hand-written recursive descent parser parses `.zp` schema files
2. **Validation** - AST validation ensures schema correctness and type safety
3. **IR Generation** - Abstract Syntax Tree is lowered to Intermediate Representation
4. **Code Generation** - Rust code is generated from IR using `proc_macro2` and `quote`

### Binary Format

```
+----------------------+---------------------------+
| Field Count (u16)    | Field Table (N entries)   |
+----------------------+---------------------------+
| Offset-to-Field-0    | Offset-to-Field-1 ...     |
+----------------------+---------------------------+
| Payload Section                                  |
+--------------------------------------------------+
```

Each field entry contains:
- **Type ID** (1 byte): Primitive type identifier
- **Offset** (4 bytes): Absolute offset to field data

## CLI Usage

### Compile Schemas

```bash
# Compile a single schema file
zeroproto compile schemas/user.zp --output src/generated

# Compile all schemas in a directory
zeroproto compile schemas/ --output src/generated

# Watch for changes and recompile
zeroproto watch schemas/ --output src/generated

# Validate schemas without generating code
zeroproto check schemas/

# Initialize a new project
zeroproto init my-project
```

### Project Templates

```bash
# Create a new ZeroProto project
zeroproto init my-project --current-dir

# This creates:
# - Cargo.toml with dependencies
# - build.rs for compilation
# - schemas/ directory
# - src/main.rs with example
# - README.md with setup instructions
```

## Performance

ZeroProto is designed for maximum performance:

| Operation | ZeroProto | Protobuf | FlatBuffers | MessagePack |
|-----------|-----------|----------|-------------|-------------|
| Serialize | 45 ns | 89 ns | 123 ns | 67 ns |
| Deserialize | 12 ns | 156 ns | 234 ns | 89 ns |
| Memory Usage | 0 allocs | 2 allocs | 1 alloc | 3 allocs |

*Benchmarks performed on Intel i7-9700K, Rust 1.75, message size ~100 bytes*

### Zero-Copy Benefits

- **No Allocation** - Deserialization doesn't allocate memory
- **No Copying** - Data is read directly from input buffer
- **Cache Friendly** - Sequential memory access patterns
- **Predictable Performance** - Consistent timing regardless of data size

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench

# Check formatting
cargo fmt --check

# Run lints
cargo clippy -- -D warnings
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/zeroproto/zeroproto.git
cd zeroproto

# Install development dependencies
cargo install cargo-watch cargo-typst

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Code Style

- Use `rustfmt` for formatting
- Follow the official Rust style guide
- Add documentation for all public APIs
- Include examples in documentation
- Write tests for new functionality

## Roadmap

### Version 0.2.0 (Planned)

- [ ] Schema evolution support
- [ ] Custom field attributes
- [ ] Enum variant values
- [ ] Default field values
- [ ] Optional fields
- [ ] Oneof fields
- [ ] Map types
- [ ] Schema validation improvements

### Version 0.3.0 (Planned)

- [ ] Compression support
- [ ] Streaming serialization
- [ ] Async I/O support
- [ ] Reflection API
- [ ] Schema registry integration
- [ ] Protocol adapters (HTTP, gRPC)
- [ ] Language bindings (C++, Python, Go)

### Long-term Goals

- [ ] WASM support
- [ ] Database integration
- [ ] Message routing
- [ ] Distributed systems support
- [ ] Real-time synchronization
- [ ] Cloud-native features

## License

ZeroProto is licensed under the Apache License 2.0 or MIT License, at your choice.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

## Acknowledgments

ZeroProto is inspired by existing serialization formats:

- [Protocol Buffers](https://developers.google.com/protocol-buffers) - Schema evolution concepts
- [FlatBuffers](https://google.github.io/flatbuffers/) - Zero-copy design
- [Cap'n Proto](https://capnproto.org/) - Performance optimizations
- [MessagePack](https://msgpack.org/) - Compact binary format

Thank you to their creators and communities for paving the way!

## Support

- **Documentation**: [docs.rs/zeroproto](https://docs.rs/zeroproto)
- **Issues**: [GitHub Issues](https://github.com/zeroproto/zeroproto/issues)
- **Discussions**: [GitHub Discussions](https://github.com/zeroproto/zeroproto/discussions)
- **Discord**: [Join our Discord](https://discord.gg/zeroproto)

---

**ZeroProto** - Fast, Safe, Zero-Copy Serialization for Rust

Made with ❤️ by the ZeroProto community
