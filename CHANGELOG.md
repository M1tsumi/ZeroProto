# Changelog

Hey! Here's what's been happening with ZeroProto. We try to keep this updated so you always know what's new, what's fixed, and what might break your code.

We follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-12-02

This is a big one! We've been working hard on making ZeroProto more flexible and powerful while keeping the blazing-fast performance you love.

### New Features

- **Optional Fields** - Fields can now be marked as optional with `?` syntax. No more wrapping everything in nested messages just to handle missing data.
  ```zp
  message User {
      user_id: u64;
      nickname: string?;  // This field is optional!
      avatar_url: string?;
  }
  ```

- **Default Values** - Specify default values for fields. When a field isn't set, you get the default instead of an error.
  ```zp
  message Config {
      max_connections: u32 = 100;
      timeout_ms: u64 = 5000;
      debug_mode: bool = false;
  }
  ```

- **Map Types** - Finally! Native support for key-value maps. Keys must be primitives or strings.
  ```zp
  message UserCache {
      users: map<u64, User>;
      name_lookup: map<string, u64>;
  }
  ```

- **Oneof Fields** - When you need exactly one of several options. Great for representing variants.
  ```zp
  message Event {
      timestamp: u64;
      payload: oneof {
          user_created: UserCreatedEvent;
          user_deleted: UserDeletedEvent;
          user_updated: UserUpdatedEvent;
      };
  }
  ```

- **Streaming Serialization** - Serialize large messages in chunks without loading everything into memory. Perfect for handling big data.

- **Async I/O Support** - New async readers and writers that play nicely with Tokio and async-std.

- **Reflection API** - Inspect message structure at runtime. Useful for debugging, logging, and building generic tools.

### Performance Improvements

- **15% faster deserialization** - We reworked the field lookup logic to reduce branching
- **20% smaller generated code** - Smarter code generation means less bloat in your binaries
- **Reduced memory footprint** - Builders now use a more compact internal representation
- **Better cache utilization** - Reorganized data layout for improved cache locality

### Developer Experience

- **Improved error messages** - When something goes wrong, you'll actually understand why
- **Better IDE support** - Enhanced proc-macro hygiene for better rust-analyzer integration
- **Schema validation warnings** - Catch potential issues before they become problems
- **New `--verbose` flag** - See exactly what the compiler is doing

### Bug Fixes

- Fixed a rare panic when deserializing deeply nested messages (thanks @community-member!)
- Resolved an issue where empty vectors weren't handled correctly in some edge cases
- Fixed code generation for enum variants with large discriminant values
- Corrected lifetime handling in generated reader code for complex nested types

### Breaking Changes

- Minimum Rust version is now 1.75 (was 1.70)
- `MessageReader::new()` now returns `Result` instead of panicking on invalid data
- Renamed `VectorReader::count()` to `VectorReader::len()` for consistency with std

### Migration Guide

Upgrading from 0.2.x? Here's what you need to know:

1. Update your `Cargo.toml` to use version `0.3.0`
2. If you were calling `MessageReader::new()`, wrap it in error handling
3. Replace any `.count()` calls on vectors with `.len()`
4. Regenerate your schema code with `zeroproto compile`

That's it! Most code should work without changes.

---

## [0.2.0] - 2025-11-30

This release was all about laying a solid foundation. We rewrote the parser from scratch and cleaned up a lot of technical debt.

### What Changed

- **New Parser** - We ditched LALRPOP and wrote our own recursive descent parser. It's faster, produces better error messages, and has zero external dependencies.
- **Better Error Messages** - When your schema has a problem, you'll now get helpful context about where and why.
- **Reserved Name Checking** - We now warn you if you try to use field names like `id`, `type`, `data`, or `buffer` that could cause issues.
- **Cleaner Codegen** - Fixed a bunch of inconsistencies in the generated code.

### Bug Fixes

- Fixed template variable issues in enum code generation
- Corrected field type handling between `rust_name` and `rust_type`
- Fixed type signature mismatches in generic path parameters
- Resolved missing imports and module declarations
- Fixed parser handling for comma-separated field lists
- Corrected AST to IR lowering for user-defined types

### New Stuff

- Added `primitives` module with `PrimitiveType` enum
- Parser now accepts optional trailing commas in field lists
- Comprehensive reserved name validation
- Better error handling throughout the compilation pipeline
- Improved CLI feedback

### Removed

- LALRPOP dependency (good riddance to the build script!)
- Dead code and unused imports

### Performance

- Faster compilation thanks to the simpler parser
- Smaller dependency tree
- Better memory efficiency during parsing
- Improved runtime serialization/deserialization speed

## [0.1.0] - 2024-11-29

The beginning! This was our first public release.

### What We Shipped

- **Core Runtime** - `MessageReader` and `MessageBuilder` for zero-copy serialization
- **Schema Compiler** - Parse `.zp` files and generate type-safe Rust code
- **CLI Tool** - `compile`, `watch`, `check`, and `init` commands
- **Proc Macros** - Derive macros for automatic reader/builder generation
- **Full Documentation** - README, API docs, format spec, and examples
- **Test Suite** - Unit tests, integration tests, and benchmarks
- **no_std Support** - Works in embedded environments out of the box

### The Runtime Library

- `MessageReader` for zero-copy deserialization
- `MessageBuilder` for serialization
- `VectorReader` and `VectorBuilder` for collections
- All primitive types supported
- Memory-safe buffer handling

### The Compiler

- Hand-written recursive descent parser
- AST validation and type checking
- IR generation and Rust code output
- Incremental build support
- Detailed error diagnostics

### The CLI

- `zeroproto compile` - Generate code from schemas
- `zeroproto watch` - Auto-recompile on file changes
- `zeroproto check` - Validate schemas without generating code
- `zeroproto init` - Scaffold new projects

---

## How We Version

We use [Semantic Versioning](https://semver.org/):

- **Major (x.0.0)** - Breaking changes. Read the migration guide!
- **Minor (0.x.0)** - New features, fully backward compatible
- **Patch (0.0.x)** - Bug fixes only

### What We Promise

- Your serialized data will work across all versions with the same major number
- The schema language won't break between minor versions
- We'll always provide migration guides for breaking changes
- Security fixes get backported to supported versions

### When We Release

- **Major releases**: Every 6-12 months (big features, possible breaking changes)
- **Minor releases**: Every 1-3 months (new features, improvements)
- **Patch releases**: Whenever needed (bug fixes, security updates)

---

## Platform Support

| Tier | Platforms | What It Means |
|------|-----------|---------------|
| 1 | Linux, macOS, Windows | Fully tested, guaranteed to work |
| 2 | FreeBSD, NetBSD, OpenBSD | Should work, tested occasionally |
| 3 | Embedded (ARM Cortex-M, RISC-V) | Community supported, no_std compatible |

**Minimum Rust Version**: 1.75

---

## Security

Found a vulnerability? Please report it privately:

- **Email**: security@zeroproto.dev
- **GitHub**: Create a private security advisory

We take security seriously and will prioritize fixes.

---

## Want to Help?

We'd love to have you! Check out:

- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [GitHub Issues](https://github.com/zeroproto/zeroproto/issues) - Bug reports and feature requests
- [GitHub Discussions](https://github.com/zeroproto/zeroproto/discussions) - Questions and ideas

---

## License

MIT or Apache 2.0, your choice. See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
