# Changelog

All notable changes to ZeroProto will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Breaking**: Replaced LALRPOP parser with hand-written recursive descent parser
- Improved compilation performance and reduced dependencies
- Enhanced error messages with better context and location information
- Updated field name validation rules (avoid reserved names like "id", "type", "data", "buffer")
- Updated enum name validation rules (avoid reserved names like "Result", "Option", "Status")
- Fixed all compilation errors in `zeroproto-compiler` crate
- Resolved `rust_name` vs `rust_type` inconsistencies in codegen
- Fixed template variable resolution in enum generation
- Improved CLI integration and type safety

### Fixed
- Fixed template variable issues (`variant_name`, `variant_value`, `reader_name`) in code generation
- Corrected field type enum usage between `rust_name` and `rust_type`
- Fixed type signature mismatches in `lib.rs` generic path parameters
- Resolved missing imports and module declarations
- Fixed parser token handling for comma-separated field lists
- Corrected AST to IR lowering for user-defined types

### Added
- Added `primitives` module with `PrimitiveType` enum
- Enhanced parser with support for optional commas in field lists
- Improved validation with comprehensive reserved name checking
- Added proper error handling throughout the compilation pipeline
- Enhanced CLI with better error reporting and user feedback

### Removed
- Removed LALRPOP dependency and build script
- Removed unused imports and dead code
- Simplified parser implementation for better maintainability

## [0.1.0] - 2024-11-29

### Added
- Initial release of ZeroProto
- Core serialization and deserialization functionality
- Schema language and compiler
- CLI tool with compile, watch, and check commands
- Project initialization templates
- Comprehensive test coverage
- Performance benchmarks
- Full documentation suite

### Features
- **Runtime Library** (`zeroproto`)
  - `MessageReader` for zero-copy deserialization
  - `MessageBuilder` for serialization
  - `VectorReader` and `VectorBuilder` for collections
  - Support for all primitive types
  - Memory-safe buffer handling
  - no_std compatibility

- **Schema Compiler** (`zeroproto-compiler`)
  - Hand-written recursive descent parser for `.zp` schema files
  - AST validation and type checking
  - Intermediate representation (IR) generation
  - Rust code generation with type-safe APIs
  - Incremental build support
  - Error reporting with detailed diagnostics

- **Procedural Macros** (`zeroproto-macros`)
  - `ZeroprotoMessage` derive macro for message types
  - `ZeroprotoFields` derive macro for field accessors
  - Automatic reader and builder generation
  - Type-safe field access methods

- **CLI Tool** (`zeroproto-cli`)
  - `compile` command for schema compilation
  - `watch` command for automatic recompilation
  - `check` command for schema validation
  - `init` command for project scaffolding
  - Verbose output and error reporting

### Documentation
- Comprehensive README with quick start guide
- Binary format specification document
- API documentation with examples
- Performance benchmarks and comparisons
- Contributing guidelines and code of conduct
- Integration examples and best practices

### Testing
- Unit tests for all core components
- Integration tests for end-to-end workflows
- Property-based tests for edge cases
- Performance benchmarks with Criterion
- Memory safety tests
- Cross-platform compatibility tests

### Examples
- Basic usage examples
- Complex nested message examples
- Vector and collection handling
- Error handling patterns
- Performance optimization examples

### Benchmarks
- Roundtrip serialization/deserialization performance
- Memory usage comparisons
- Zero-copy vs copy-based deserialization
- Large message handling performance
- Vector serialization benchmarks

### Development Tools
- Workspace configuration for multi-crate development
- Automated testing and benchmarking
- Documentation generation
- Release automation
- CI/CD configuration templates

### Quality Assurance
- 100% documentation coverage for public APIs
- Comprehensive error handling
- Memory safety guarantees
- No unsafe code in public APIs
- Strict linting rules and formatting

---

## Version Policy

ZeroProto follows [Semantic Versioning](https://semver.org/):

- **Major versions** introduce breaking changes
- **Minor versions** add new features in a backward-compatible manner  
- **Patch versions** fix bugs in a backward-compatible manner

### Compatibility Guarantees

- Serialized data format is stable within major versions
- Schema language is backward compatible within major versions
- Public API changes only occur in major versions
- Configuration and CLI options remain stable within major versions

### Deprecation Policy

- Deprecated features will be announced in release notes
- Deprecated features will be removed in the next major version
- Migration guides will be provided for breaking changes
- Security fixes will be backported to supported versions

---

## Release Schedule

ZeroProto aims for regular releases with the following schedule:

- **Major releases**: Every 6-12 months for significant features
- **Minor releases**: Every 1-3 months for new features and improvements
- **Patch releases**: As needed for bug fixes and security updates

### Release Process

1. All changes must pass CI/CD checks
2. Documentation must be updated
3. Changelog must be updated
4. Version numbers must be updated
5. Release notes must be written
6. Packages are published to crates.io
7. GitHub releases are created
8. Documentation is deployed

---

## Supported Platforms

ZeroProto supports the following platforms:

- **Tier 1**: Linux, macOS, Windows (latest stable versions)
- **Tier 2**: FreeBSD, NetBSD, OpenBSD
- **Tier 3**: Embedded targets (ARM Cortex-M, RISC-V, etc.)

### Rust Version Support

- Minimum supported Rust version: 1.75
- Recommended Rust version: latest stable
- Tested with: 1.75, 1.76, 1.77, 1.78

---

## Security

Security vulnerabilities should be reported privately to:

- Email: security@zeroproto.dev
- GitHub: Create a private security advisory

Security fixes will be:

- Prioritized over other changes
- Backported to supported versions
- Released as patch versions
- Documented in security advisories

---

## Contributing

Contributions are welcome! Please see:

- [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines
- [Code of Conduct](CODE_OF_CONDUCT.md) for community standards
- GitHub Issues for bug reports and feature requests
- GitHub Discussions for questions and ideas

---

## License

ZeroProto is licensed under the Apache License 2.0 or MIT License, at your choice.

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
