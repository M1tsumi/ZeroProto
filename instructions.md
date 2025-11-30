# ZeroProto Instructions

## Overview

ZeroProto is a Rust-first binary messaging and serialization format designed for:

* **Zero-copy deserialization** (borrow directly from underlying byte buffer)
* **Compile-time code generation** from `.zp` schema files
* **High performance** alternative to Protobuf, FlatBuffers, and Cap’n Proto
* **Minimal allocations** and **dense** binary layout
* **Seamless integration with Rust’s borrow checker**

ZeroProto provides:

* A schema language (`.zp`) describing messages, enums, and structures
* A compiler (`zeroproto-compiler`) that generates Rust types, serializers, and readers
* A runtime crate (`zeroproto`) offering read/write primitives, buffer abstractions, and safety utilities

This file outlines everything needed to fully implement the ZeroProto system.

---

# 1. Project Structure

```
zeroproto/
 ├─ crates/
 │   ├─ zeroproto/                # Runtime library
 │   ├─ zeroproto-compiler/       # Schema compiler & codegen
 │   ├─ zeroproto-macros/         # Optional proc macros
 │   └─ zeroproto-cli/            # Command-line interface
 │
 ├─ docs/
 │   └─ specifications.md         # Binary format specification
 │
 ├─ schemas/                      # Example .zp schemas
 └─ tests/
```

---

# 2. Core Concepts

## 2.1 Zero-Copy Deserialization

ZeroProto ensures that deserialized fields referencing strings, bytes, or nested structures:

* Do **not** allocate
* Borrow directly from the source byte slice (`&[u8]`)
* Maintain safety through bounds-checked slices

**Design principle:** All message offsets are absolute from start of message.

## 2.2 Schema-Defined Layout

ZeroProto layouts are fully determined by the `.zp` schema. Fields may be:

* **Scalar** (`u8`, `u16`, `u32`, `i64`, `f32`, etc.)
* **Slices** (`bytes`, `string`)
* **Vectors** (`[Type]`)
* **Nested messages**

## 2.3 Auto Code Generation

The compiler reads `.zp` files and outputs Rust modules:

* `struct` for each message
* compile-time verified offsets
* serializer + builder API
* reader API with lifetimes

---

# 3. Schema Language (`.zp`)

## 3.1 Example

```
message User {
  id: u64;
  username: string;
  friends: [u64];
  profile: Profile;
}

message Profile {
  bio: string;
  age: u8;
}
```

## 3.2 Grammar (high-level)

```
file        := (message | enum)*
message     := 'message' IDENT '{' field* '}'
field       := IDENT ':' type ';'
type        := scalar | IDENT | '[' type ']'
scalar      := 'u8' | 'u16' | ... | 'f64' | 'bool' | 'string' | 'bytes'
```

## 3.3 Supported Scalar Types

* Unsigned: `u8`, `u16`, `u32`, `u64`
* Signed: `i8`, `i16`, `i32`, `i64`
* Floats: `f32`, `f64`
* Boolean: `bool`
* Text/Binary: `string`, `bytes`

## 3.4 Reserved Keywords

```
message enum string bytes true false
```

---

# 4. Binary Format Specification

Located in detail in `docs/specifications.md`. Summary:

## 4.1 Message Layout

Each message is encoded as:

```
+----------------------+---------------------------+
| Field Count (u16)    | Field Table (N entries)   |
+----------------------+---------------------------+
| Offset-to-Field-0    | Offset-to-Field-1 ...     |
+----------------------+---------------------------+
| Payload Section                                  |
+--------------------------------------------------+
```

## 4.2 Field Table Entry

Each field entry contains:

```
struct FieldEntry {
  type_id: u8;
  offset: u32; // absolute offset from message start
}
```

## 4.3 Payload Encodings

* Scalars: fixed-width
* Slices: stored as `len: u32` + raw bytes
* Nested messages: recursively encoded
* Vectors: `count: u32` then repeated payloads

---

# 5. Runtime (`zeroproto` crate)

## Responsibilities

* Provide reader types (`MessageReader<'a>`, `VectorReader<'a>`, etc.)
* Provide builders (`MessageBuilder`, `VectorBuilder`)
* Safety + bounds checking
* Endianness handling (little-endian)

## Modules

```
zeroproto/
 ├─ reader.rs
 ├─ builder.rs
 ├─ errors.rs
 ├─ primitives.rs
 ├─ vector.rs
 └─ macros.rs (optional)
```

## 5.1 Reader API

```
impl<'a> UserReader<'a> {
    fn id(&self) -> u64;
    fn username(&self) -> &'a str;
    fn friends(&self) -> VectorReader<'a, u64>;
    fn profile(&self) -> ProfileReader<'a>;
}
```

## 5.2 Builder API

```
let mut b = UserBuilder::new();
b.set_id(123);
b.set_username("Thomas");
b.add_friend(77);
let bytes = b.finish();
```

---

# 6. Compiler (`zeroproto-compiler`)

## Responsibilities

* Parse `.zp` files
* Validate messages and types
* Create an intermediate representation (IR)
* Generate Rust code
* Output modules with:

  * `Reader` structs
  * `Builder` structs
  * Field offset constants
  * Type-safe vector/array helpers

## 6.1 Architecture

```
compiler/
 ├─ parser.rs       # Tokenizer + AST builder
 ├─ ir.rs           # Intermediate representation
 ├─ validator.rs    # Type & layout checking
 ├─ codegen/
 │    ├─ rust_reader.rs
 │    ├─ rust_builder.rs
 │    └─ mod.rs
 └─ main.rs         # CLI entrypoint
```

## 6.2 Code Generation Rules

* Every message produces: `FooReader`, `FooBuilder`
* Offsets generated as `const FIELD_0_OFFSET: usize`
* Lifetimes encoded automatically: `'a`
* Nested vectors produce nested readers

---

# 7. CLI (`zeroproto-cli`)

## Commands

```
zeroproto compile path/to/schema.zp -o src/generated
zeroproto watch schemas/ -o src/generated
zeroproto check schema.zp
```

## Features

* File watching with incremental rebuild
* Pretty diagnostics
* Auto `mod.rs` generation
* Verbose mode

---

# 8. Integration into a Rust Project

## Add to `Cargo.toml`

```
[dependencies]
zeroproto = { path = "../crates/zeroproto" }

[build-dependencies]
zeroproto-compiler = { path = "../crates/zeroproto-compiler" }
```

## Build Script (`build.rs`)

```
fn main() {
    zeroproto_compiler::compile("schemas/user.zp", "src/generated").unwrap();
}
```

## Importing Generated Code

```
mod generated;
use generated::user::*;
```

---

# 9. Testing Strategy

## Unit Tests

* Primitive read/write tests
* Bounds checking
* String validation
* Vector serialization/deserialization

## Integration Tests

* End-to-end message encode → decode
* Fuzzing of malformed buffers

## Benchmarking

Use Criterion:

```
criterion_group!(benches, bench_user_roundtrip);
```

---

# 10. Style & Quality Guidelines

* All unsafe operations must be wrapped internally
* Public API must be fully documented
* Include examples in `///` docs
* Avoid panics for normal error conditions
* Prefer compile-time verification

---

# 11. Deliverables

A complete implementation must include:

* **Fully working runtime crate**
* **Schema parser + validator + code generator**
* **CLI tool**
* **Extensive documentation**
* **Benchmarks**
* **Test suite**
* **Examples**

All modules should follow Rust 2024 edition idioms.

---

# 12. Future Extensions

* C and TypeScript codegen
* Schema evolution rules
* Optional compression blocks
* Self-describing messages
* RPC layer

---

# End of ZeroProto Instructions
