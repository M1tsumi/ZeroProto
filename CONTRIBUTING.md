# Contributing to ZeroProto

Thank you for your interest in contributing to ZeroProto! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Git
- Basic knowledge of Rust and serialization concepts

### Development Setup

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/your-username/zeroproto.git
   cd zeroproto
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/zeroproto/zeroproto.git
   ```
4. Install development dependencies:
   ```bash
   cargo install cargo-watch cargo-typst
   ```

### Building and Testing

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-fix-name
```

### 2. Make Changes

- Follow the existing code style and conventions
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass

### 3. Commit Changes

Use clear and descriptive commit messages:

```
feat: add support for enum types in schema
fix: resolve buffer overflow in vector parsing
docs: update README with installation instructions
test: add integration tests for nested messages
```

### 4. Submit Pull Request

1. Push your branch:
   ```bash
   git push origin feature/your-feature-name
   ```
2. Open a pull request on GitHub
3. Fill out the pull request template
4. Wait for code review

## Code Style Guidelines

### Rust Conventions

- Use `rustfmt` for formatting
- Follow the official Rust style guide
- Use meaningful variable and function names
- Add `#[allow(dead_code)]` sparingly and with justification

### Documentation

- All public items must have documentation
- Use `///` for item documentation
- Include examples in documentation
- Use `#[doc]` attributes for additional metadata

### Error Handling

- Use `Result<T, Error>` for fallible operations
- Provide meaningful error messages
- Handle errors gracefully
- Use `?` operator for error propagation

## Architecture Guidelines

### Core Principles

1. **Zero-copy**: Maintain zero-copy deserialization
2. **Safety**: Ensure memory safety at all times
3. **Performance**: Optimize for speed and memory usage
4. **Simplicity**: Keep the API simple and intuitive

### Module Organization

- `crates/zeroproto`: Core runtime library
- `crates/zeroproto-compiler`: Schema compiler and codegen
- `crates/zeroproto-cli`: Command-line interface
- `crates/zeroproto-macros`: Procedural macros

### Adding New Features

1. Consider the impact on zero-copy deserialization
2. Ensure backward compatibility when possible
3. Add comprehensive tests
4. Update documentation and examples

## Testing Guidelines

### Test Types

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test end-to-end functionality
- **Benchmark tests**: Measure performance
- **Property tests**: Test with random inputs

### Test Organization

```
tests/
├── integration_tests.rs    # End-to-end tests
├── compiler_tests.rs       # Compiler-specific tests
└── parser_tests.rs         # Parser-specific tests

crates/zeroproto/tests/     # Runtime library tests
crates/zeroproto-compiler/tests/  # Compiler tests
```

### Writing Tests

- Use descriptive test names
- Test both success and error cases
- Use `#[should_panic]` for panic tests
- Mock external dependencies when needed

## Documentation Standards

### README.md

- Keep it up-to-date
- Include installation instructions
- Provide quick start examples
- Link to additional documentation

### API Documentation

- Document all public APIs
- Include usage examples
- Explain performance characteristics
- Note any safety considerations

### Schema Documentation

- Document schema language features
- Provide examples for each construct
- Explain type system rules
- Include best practices

## Release Process

### Versioning

ZeroProto follows [Semantic Versioning](https://semver.org/):

- **Major**: Breaking changes
- **Minor**: New features (backward compatible)
- **Patch**: Bug fixes (backward compatible)

### Release Checklist

1. Update version numbers
2. Update CHANGELOG.md
3. Run full test suite
4. Update documentation
5. Create git tag
6. Publish to crates.io
7. Update GitHub releases

## Community Guidelines

### Code of Conduct

Be respectful, inclusive, and professional. Follow the [Rust Code of Conduct](https://www.rust-lang.org/conduct.html).

### Communication

- Use GitHub issues for bug reports and feature requests
- Use GitHub discussions for general questions
- Be patient and helpful with new contributors

### Getting Help

- Read the documentation first
- Search existing issues and discussions
- Ask questions in GitHub discussions
- Join our Discord server (link in README)

## Performance Guidelines

### Benchmarking

- Use Criterion for benchmarks
- Test with realistic data sizes
- Compare against baseline measurements
- Profile for optimization opportunities

### Memory Usage

- Minimize allocations
- Use stack allocation when possible
- Profile memory usage patterns
- Consider cache efficiency

### Optimization

- Profile before optimizing
- Focus on hot paths
- Consider compiler optimizations
- Test performance regressions

## Security Considerations

### Input Validation

- Validate all input data
- Check buffer bounds
- Handle malformed data gracefully
- Prevent buffer overflows

### Memory Safety

- Use safe Rust when possible
- Document unsafe code thoroughly
- Audit unsafe code regularly
- Consider formal verification for critical components

## Contributing Tools

### IDE Configuration

#### VS Code

Install these extensions:
- rust-analyzer
- CodeLLDB
- Better TOML

#### Configuration

```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "rust-analyzer.procMacro.enable": true
}
```

### Git Hooks

Use pre-commit hooks to ensure code quality:

```bash
#!/bin/sh
# pre-commit hook
cargo fmt --all
cargo clippy -- -D warnings
cargo test
```

## Reporting Issues

### Bug Reports

Include:
- Rust version
- Operating system
- Minimal reproducible example
- Expected vs actual behavior
- Backtrace if available

### Feature Requests

Include:
- Use case description
- Proposed API design
- Implementation considerations
- Alternative approaches

## Acknowledgments

Thank you to all contributors who make ZeroProto better! Your contributions are greatly appreciated.

### Contributors

- Add your name here when you make your first contribution

### Inspiration

ZeroProto is inspired by existing serialization formats like:
- Protocol Buffers
- FlatBuffers
- Cap'n Proto
- MessagePack

Thank you to their creators and communities for paving the way!
