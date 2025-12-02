# ZeroProto Binary Format Specification

This document explains how ZeroProto encodes data on the wire. If you're just using ZeroProto, you don't need to read this - the library handles everything. But if you're curious about the internals, implementing a parser in another language, or debugging weird issues, read on!

## The Big Picture

Every ZeroProto message has three parts:

1. **Header** - How many fields are in this message
2. **Field Table** - Where each field lives and what type it is
3. **Payload** - The actual data

## Message Layout

Here's what a message looks like in memory:

```
+----------------------+---------------------------+
| Field Count (u16)    | Field Table (N entries)   |
+----------------------+---------------------------+
| Offset-to-Field-0    | Offset-to-Field-1 ...     |
+----------------------+---------------------------+
| Payload Section                                  |
+--------------------------------------------------+
```

### The Header

Just 2 bytes telling us how many fields to expect:

- **Field Count** (u16): 0 to 65,535 fields per message

### The Field Table

Each field gets a 5-byte entry:

```
+------------------+------------------+
| Type ID (1 byte) | Offset (4 bytes) |
+------------------+------------------+
```

- **Type ID**: What kind of data this is (see table below)
- **Offset**: Where to find the data, measured from the start of the message

This is what makes zero-copy possible - we can jump directly to any field without scanning through the whole message.

## Type IDs

Every field has a type ID that tells us how to read it:

| ID | Type | Size | Notes |
|----|------|------|-------|
| 0 | u8 | 1 byte | Unsigned byte |
| 1 | u16 | 2 bytes | Unsigned short |
| 2 | u32 | 4 bytes | Unsigned int |
| 3 | u64 | 8 bytes | Unsigned long |
| 4 | i8 | 1 byte | Signed byte |
| 5 | i16 | 2 bytes | Signed short |
| 6 | i32 | 4 bytes | Signed int |
| 7 | i64 | 8 bytes | Signed long |
| 8 | f32 | 4 bytes | Float |
| 9 | f64 | 8 bytes | Double |
| 10 | bool | 1 byte | 0 = false, non-zero = true |
| 11 | string | variable | Length-prefixed UTF-8 |
| 12 | bytes | variable | Length-prefixed raw bytes |
| 13 | message | variable | Nested ZeroProto message |
| 14 | vector | variable | Array of values |

## How Data Is Encoded

### Numbers (Scalars)

All numbers are little-endian. Simple and consistent:

| Type | Bytes |
|------|-------|
| u8, i8, bool | 1 |
| u16, i16 | 2 |
| u32, i32, f32 | 4 |
| u64, i64, f64 | 8 |

### Strings and Bytes

Length-prefixed, nothing fancy:

```
+------------------+------------------+
| Length (u32)     | Data...          |
+------------------+------------------+
```

For strings, the data is UTF-8 encoded. For bytes, it's raw.

### Nested Messages

Same idea - length prefix, then the message:

```
+------------------+------------------+
| Length (u32)     | Message...       |
+------------------+------------------+
```

The nested message is a complete ZeroProto message with its own header and field table.

### Vectors

Count prefix, then elements packed together:

```
+------------------+------------------+------------------+
| Count (u32)      | Element 0        | Element 1 ...    |
+------------------+------------------+------------------+
```

For fixed-size types (numbers), elements are packed directly. For variable-size types (strings, messages), each element has its own length prefix.

## No Padding, No Alignment

We pack everything as tightly as possible. No wasted bytes for alignment. This keeps messages small but means you can't just cast a pointer to a struct (not that you'd want to in safe Rust anyway).

## Always Little-Endian

Every multi-byte value is little-endian. This is the native byte order on x86/x64 and ARM, so it's fast on most hardware.

## A Real Example

Let's trace through an actual message. Given this schema:

```zp
message User {
    user_id: u64;
    name: string;
    age: u8;
}
```

With values: `user_id = 12345`, `name = "Alice"`, `age = 30`

Here's the byte-by-byte breakdown:

```
Offset  Bytes                     Meaning
------  ------------------------  --------------------------------
0x00    03 00                     Field count = 3
0x02    03 11 00 00 00            Field 0: type=3 (u64), offset=17
0x07    0B 19 00 00 00            Field 1: type=11 (string), offset=25
0x0C    00 22 00 00 00            Field 2: type=0 (u8), offset=34
0x11    39 30 00 00 00 00 00 00   user_id = 12345
0x19    05 00 00 00               name length = 5
0x1D    41 6C 69 63 65            "Alice" in UTF-8
0x22    1E                        age = 30
```

Total: 35 bytes. Not bad for a user record!

## Validation Rules

When reading a message, we check:

### Message-Level
- Field count ≤ 65,535
- All offsets point inside the message
- Offsets are strictly increasing (no overlaps)
- Field table fits in the message

### Field-Level
- Type IDs are valid (0-14)
- String/vector lengths don't exceed remaining buffer
- Nested messages are valid ZeroProto messages
- UTF-8 strings are actually valid UTF-8

### What We Don't Allow
- Nested vectors (vectors of vectors) - use a wrapper message instead
- Circular references - messages can't contain themselves

## Why It's Fast

### Zero-Copy Magic

When you read a string field, you get a `&str` that points directly into the input buffer. No copying, no allocation. The Rust lifetime system ensures you can't use the string after the buffer is gone.

### Cache-Friendly

- The field table is small (5 bytes per field) and usually fits in L1 cache
- Field data is accessed sequentially
- Minimal pointer chasing

### Compact

- No padding bytes
- Variable-length encoding for strings/vectors
- Overhead is just 2 bytes + 5 bytes per field

## Compatibility

### Schema Evolution

| Change | Safe? | Notes |
|--------|-------|-------|
| Add new field at end | Yes | Old readers ignore it |
| Remove field | No | Old readers will fail |
| Change field type | No | Type mismatch error |
| Rename field | Yes | Names aren't in the binary |
| Reorder fields | No | Field order matters |

### Cross-Platform

- Little-endian everywhere (no byte-swapping needed on x86/ARM)
- Fixed-size integers (no "int is 32-bit on this platform" issues)
- UTF-8 strings (universal text encoding)

## Security

ZeroProto is designed to handle untrusted input safely:

### What We Check
- All offsets are bounds-checked before use
- String/vector lengths are validated
- We never trust the data to be well-formed

### What Rust Gives Us
- No buffer overflows (bounds checking)
- No use-after-free (lifetime tracking)
- No null pointer dereferences (Option types)

### Fuzzing

We fuzz-test the parser regularly. If you find a way to crash it with malformed input, please report it!

## Implementation Notes

If you're implementing a ZeroProto parser:

### Reading

1. Read field count (first 2 bytes)
2. Read field table (5 bytes × field count)
3. Validate all offsets are in bounds
4. For each field access, look up offset in table, read data at that offset

### Writing

1. Collect all field data
2. Calculate offsets (header size + field table size + cumulative data sizes)
3. Write header
4. Write field table
5. Write payload

### Error Handling

- Never panic on malformed input
- Return descriptive errors
- Include byte offsets in error messages when possible

---

Questions? Open an issue on GitHub or ask in Discord!
