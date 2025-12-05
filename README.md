# ZeroProto

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-APACHE)
[![Discord](https://img.shields.io/discord/1302036475148349453?label=Discord&logo=discord)](https://discord.gg/6nS2KqxQtj)
[![Support me on Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/quefep)

Hey there! If you're tired of slow serialization eating into your app's performance, you're in the right place. ZeroProto is a **zero-copy binary serialization library** built from the ground up for Rust developers who care about speed.

The idea is simple: instead of copying data around when you deserialize, ZeroProto reads directly from the original buffer. No allocations, no copying, just raw speed. And because everything is generated from schema files, you get full type safety at compile time.

## Why ZeroProto?

We built ZeroProto because existing solutions either sacrificed performance for convenience or were a pain to work with. Here's what makes it different:

- **Zero-Copy Deserialization** - Your data stays where it is. We just read it in place.
- **Schema-First Design** - Define your messages in `.zp` files, get type-safe Rust code.
- **Compile-Time Safety** - Catch errors before your code even runs.
- **Blazing Fast** - We're talking nanoseconds, not microseconds.
- **Memory Safe** - No unsafe code in public APIs. Sleep well at night.
- **Embedded Ready** - Full `no_std` support for resource-constrained environments.
- **Cross-Platform** - Consistent little-endian format everywhere.
- **Rich Types** - Primitives, strings, bytes, vectors, nested messages, enums—we've got you covered.

## Getting Started

Let's get you up and running in under 5 minutes.

### Installation

Add these to your `Cargo.toml`:

```toml
[dependencies]
zeroproto = "0.4.0"

[build-dependencies]
zeroproto-compiler = "0.4.0"
```

### Define Your Schema

Create a `schemas/user.zp` file. This is where you describe your data structures:

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

### Set Up Code Generation

Create a `build.rs` file in your project root. This tells Cargo to compile your schemas during the build process:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    zeroproto_compiler::build()?;
    Ok(())
}
```

### Use Your Generated Types

Now for the fun part—actually using your types:

```rust
mod generated;
use generated::user::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a user message
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
    
    // Read it back—this is where the magic happens!
    // No copying, no allocations. Just direct buffer access.
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

## Learn More

- [API Documentation](https://docs.rs/zeroproto) - Full API reference
- [Binary Format Specification](docs/specifications.md) - How the wire format works
- [Schema Language Guide](docs/schema-guide.md) - Everything about `.zp` files
- [Performance Benchmarks](docs/benchmarks.md) - Numbers don't lie
- [Migration Guide](docs/migration.md) - Upgrading between versions

## How It Works

ZeroProto is organized as a workspace with four crates:

- **`zeroproto`** - The runtime library. Readers, builders, and all the core types.
- **`zeroproto-compiler`** - Parses your `.zp` schemas and generates Rust code.
- **`zeroproto-macros`** - Procedural macros for derive support.
- **`zeroproto-cli`** - Command-line tool for compiling, watching, and validating schemas.

### The Compilation Pipeline

When you compile a schema, here's what happens under the hood:

1. **Parsing** - A hand-written recursive descent parser reads your `.zp` files
2. **Validation** - We check for errors, type mismatches, and reserved names
3. **IR Generation** - The AST gets lowered to an intermediate representation
4. **Code Generation** - Finally, we emit clean Rust code using `proc_macro2` and `quote`

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

The CLI makes working with schemas a breeze.

### Inspecting & Filtering Schemas

Need a quick snapshot of what’s inside a schema tree? Pair `--include/--exclude` with the new `inspect` command to slice data any way you like:

```bash
# Only look at schemas for tenant-a while skipping deprecated folders
zeroproto inspect schemas/ \
    --include "tenants/tenant-a/**/*.zp" \
    --exclude "**/deprecated/**" \
    --verbose
```

Example output:

```
📄 schemas/tenants/tenant-a/profile.zp
   Messages: 3 | Enums: 1
   Fields: 18 (optional 6, defaults 4, vectors 3)
     • msg Profile — fields: 7, optional: 2, defaults: 1, vectors: 1
     • enum Theme — variants: 3

📊 Inspection Summary
   Files: 2
   Messages: 5
   Enums: 2
   Fields: 31 (optional 9, defaults 6, vectors 5)
```

The same filters apply to `compile`, `watch`, and `check`, and every run prints which files were included vs skipped so you can validate coverage in large workspaces.

### Compiling Schemas

```bash
# Compile a single file
zeroproto compile schemas/user.zp --output src/generated

# Compile everything in a directory
zeroproto compile schemas/ --output src/generated

# Watch mode—recompiles automatically when files change
zeroproto watch schemas/ --output src/generated

# Just validate without generating code
zeroproto check schemas/

# Filter large schema trees (glob patterns are relative to the input root)
zeroproto compile schemas/ --include "tenantA/**/*.zp" --exclude "**/legacy.zp"

# Summarize schema structure without generating code
zeroproto inspect schemas/ --verbose

# Scaffold a new project
zeroproto init my-project
```

### Starting a New Project

```bash
# Create a fresh ZeroProto project
zeroproto init my-project --current-dir

# You'll get:
# - Cargo.toml with all dependencies configured
# - build.rs ready to go
# - schemas/ directory for your .zp files
# - src/main.rs with a working example
# - README.md with setup instructions
```

## Performance

We obsess over performance so you don't have to. Here's how ZeroProto stacks up:

| Operation | ZeroProto | Protobuf | FlatBuffers | MessagePack |
|-----------|-----------|----------|-------------|-------------|
| Serialize | 45 ns | 89 ns | 123 ns | 67 ns |
| Deserialize | 12 ns | 156 ns | 234 ns | 89 ns |
| Memory Usage | 0 allocs | 2 allocs | 1 alloc | 3 allocs |

*Benchmarks on Intel i7-9700K, Rust 1.75, ~100 byte messages*

### Why Zero-Copy Matters

- **No Allocations** - Deserialization doesn't touch the heap
- **No Copying** - Data is read directly from the input buffer
- **Cache Friendly** - Sequential memory access patterns keep the CPU happy
- **Predictable Latency** - Consistent timing whether you're reading 10 bytes or 10MB

## Testing

We take testing seriously. Here's how to run the suite:

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

We'd love your help making ZeroProto even better! Check out our [Contributing Guide](CONTRIBUTING.md) to get started.

### Development Setup

```bash
# Clone the repo
git clone https://github.com/zeroproto/zeroproto.git
cd zeroproto

# Install dev dependencies
cargo install cargo-watch cargo-tarpaulin

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Code Style

We keep things consistent:

- Format with `rustfmt`
- Follow the Rust style guide
- Document all public APIs
- Include examples in docs
- Write tests for new features

## Roadmap

Here's where we're headed. Want to help? Jump in!

### Version 0.4.0 (Planned)

- [ ] Streaming serialization for large messages
- [ ] Async I/O support
- [ ] Compression (LZ4, Zstd)
- [ ] Map types (`map<K, V>`)
- [ ] Oneof/union fields
- [ ] Reflection API for runtime introspection

### Future Plans

- [ ] Language bindings (C++, Python, Go, TypeScript)
- [ ] WASM support for browser environments
- [ ] Schema registry for versioned schemas
- [ ] gRPC and HTTP protocol adapters
- [ ] Database integration helpers
- [ ] Real-time sync primitives

## License

Dual-licensed under MIT or Apache 2.0—pick whichever works for you.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for the details.

## Standing on the Shoulders of Giants

ZeroProto wouldn't exist without inspiration from these amazing projects:

- [Protocol Buffers](https://developers.google.com/protocol-buffers) - Schema evolution ideas
- [FlatBuffers](https://google.github.io/flatbuffers/) - Zero-copy design principles
- [Cap'n Proto](https://capnproto.org/) - Performance optimization techniques
- [MessagePack](https://msgpack.org/) - Compact binary format concepts

Huge thanks to their creators and communities!

## Get Help

Stuck? We're here to help:

- **Discord**: [Join our community](https://discord.gg/6nS2KqxQtj) - Chat with other users and the maintainers
- **GitHub Issues**: Report bugs or request features
- **GitHub Discussions**: Ask questions and share ideas

---

**ZeroProto** - Fast, Safe, Zero-Copy Serialization for Rust

Built with care by the ZeroProto community
