# Contributing to ZeroProto

Hey, thanks for wanting to help out! Whether you're fixing a typo, squashing a bug, or building a whole new feature, we appreciate it.

This guide will get you set up and ready to contribute.

## Before You Start

You'll need:

- **Rust 1.75+** - We use some newer features
- **Git** - For version control (obviously)
- **Some Rust knowledge** - You don't need to be an expert, but familiarity helps

## Setting Up Your Dev Environment

### 1. Fork and Clone

```bash
# Fork the repo on GitHub first, then:
git clone https://github.com/YOUR-USERNAME/zeroproto.git
cd zeroproto

# Add upstream so you can pull updates
git remote add upstream https://github.com/zeroproto/zeroproto.git
```

### 2. Install Dev Tools

```bash
cargo install cargo-watch cargo-tarpaulin
```

### 3. Make Sure Everything Works

```bash
# Build it
cargo build

# Run the tests
cargo test

# Check formatting
cargo fmt --check

# Run the linter
cargo clippy -- -D warnings
```

If all that passes, you're good to go!

## The Workflow

### 1. Create a Branch

Pick a descriptive name:

```bash
git checkout -b feature/optional-fields
# or
git checkout -b fix/vector-bounds-check
```

### 2. Write Your Code

A few things to keep in mind:

- Match the existing code style (we're not picky, but consistency is nice)
- Add tests for new stuff
- Update docs if you're changing behavior
- Make sure `cargo test` passes before you push

### 3. Commit with Good Messages

We loosely follow conventional commits:

```
feat: add optional field support to schema parser
fix: handle empty vectors correctly in deserialization
docs: clarify build.rs setup in README
test: add edge case tests for nested messages
perf: optimize field lookup in MessageReader
```

### 4. Open a Pull Request

```bash
git push origin your-branch-name
```

Then head to GitHub and open a PR. We'll review it as soon as we can!

## Code Style

We're not super strict, but here's what we care about:

### Formatting

- Run `cargo fmt` before committing
- That's it. Let the tool do its job.

### Naming

- Be descriptive. `field_offset` beats `fo`.
- Follow Rust conventions (snake_case for functions, CamelCase for types)

### Documentation

- Public APIs need doc comments
- Include examples when it makes sense
- If something is tricky, explain why

### Error Handling

- Return `Result` for anything that can fail
- Make error messages actually helpful
- Use `?` for propagation, `match` when you need to handle specific cases

## Architecture

### The Golden Rules

1. **Zero-copy is sacred** - Don't break it. If your change requires allocations during deserialization, think hard about whether it's worth it.
2. **Safety first** - No unsafe code in public APIs. Period.
3. **Speed matters** - We're a performance library. Benchmark your changes.
4. **Keep it simple** - If the API is confusing, it's wrong.

### Where Things Live

```
crates/
├── zeroproto/           # Runtime library (readers, builders)
├── zeroproto-compiler/  # Schema parser and code generator
├── zeroproto-cli/       # Command-line tool
└── zeroproto-macros/    # Proc macros for derive support
```

### Adding Features

Before you start:

1. Will this break zero-copy? If yes, reconsider.
2. Is it backward compatible? If not, it needs to wait for a major version.
3. Do you have tests? You need tests.
4. Is the documentation updated? It should be.

## Testing

We take testing seriously. Here's how we organize things:

### Types of Tests

- **Unit tests** - Small, focused, test one thing
- **Integration tests** - End-to-end, test the whole pipeline
- **Benchmarks** - Make sure we're still fast
- **Property tests** - Throw random data at it, see what breaks

### Where Tests Live

```
tests/                      # Integration tests
crates/zeroproto/tests/     # Runtime unit tests
crates/zeroproto-compiler/tests/  # Compiler tests
benches/                    # Criterion benchmarks
```

### Writing Good Tests

- Name them so you know what failed: `test_vector_with_zero_elements` not `test_vector`
- Test the happy path AND the error cases
- If you fixed a bug, add a regression test

## Documentation

Good docs make everyone's life easier.

### README

- Keep it current
- Quick start should actually be quick
- Link to detailed docs for the deep stuff

### API Docs

- Every public item needs a doc comment
- Include examples that actually compile
- Mention performance implications if relevant

### Schema Docs

- Show, don't just tell
- Include complete examples
- Explain the "why" not just the "what"

## Releases

We use [Semantic Versioning](https://semver.org/):

- **Major (1.0.0)** - Breaking changes
- **Minor (0.1.0)** - New features, backward compatible
- **Patch (0.0.1)** - Bug fixes only

### Release Checklist

1. Bump version numbers in all `Cargo.toml` files
2. Update `CHANGELOG.md`
3. Run `cargo test --all-features`
4. Run `cargo clippy`
5. Create and push a git tag
6. CI handles the rest (publish to crates.io, create GitHub release)

## Community

### Be Nice

We follow the [Rust Code of Conduct](https://www.rust-lang.org/conduct.html). The short version: be respectful, be helpful, don't be a jerk.

### Getting Help

1. Check the docs first
2. Search existing issues
3. Ask in GitHub Discussions or Discord
4. Open an issue if you think you found a bug

We're friendly, we promise!

## Performance

We're a performance library. Here's how we stay fast:

### Benchmarking

- Use Criterion for all benchmarks
- Test with realistic data (not just tiny messages)
- Compare before/after for any perf-related changes
- Run `cargo bench` before submitting perf PRs

### Memory

- Allocations are the enemy
- Stack > Heap when possible
- Think about cache lines
- Profile with `heaptrack` or similar

### Optimization Tips

- Profile first, optimize second
- Focus on hot paths (deserialization is called a lot)
- Check the assembly if you're doing something tricky
- Add a benchmark for anything you optimize

## Security

We handle untrusted input. Security matters.

### Input Validation

- Never trust input data
- Always check bounds before reading
- Handle malformed data with errors, not panics
- Fuzz test new parsing code

### Memory Safety

- Safe Rust by default
- If you need `unsafe`, you need a really good reason AND a safety comment
- All unsafe code gets extra review

## Dev Tools

### VS Code Setup

Install these extensions:
- **rust-analyzer** - Essential
- **CodeLLDB** - For debugging
- **Even Better TOML** - For Cargo.toml files

Recommended settings:

```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.procMacro.enable": true
}
```

### Pre-commit Hook (Optional)

Save this as `.git/hooks/pre-commit`:

```bash
#!/bin/sh
cargo fmt --check || exit 1
cargo clippy -- -D warnings || exit 1
cargo test --lib || exit 1
```

## Reporting Issues

### Bug Reports

Help us help you. Include:

- Rust version (`rustc --version`)
- OS and version
- A minimal example that reproduces the bug
- What you expected vs what happened
- Backtrace if there's a panic

### Feature Requests

Tell us:

- What problem are you trying to solve?
- What would the API look like?
- Are there alternatives you've considered?

---

## Thank You!

Seriously, thanks for contributing. Open source runs on people like you.

Every contribution matters, whether it's:
- Fixing a typo in the docs
- Reporting a bug
- Adding a test
- Building a major feature

We appreciate all of it.
