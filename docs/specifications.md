# ZeroProto Binary Format Specification

This document specifies the binary format used by ZeroProto for message serialization and deserialization.

## Overview

ZeroProto uses a compact binary format designed for zero-copy deserialization. The format consists of:

1. **Message Header** - Field count and field table
2. **Field Table** - Type and offset information for each field
3. **Payload Section** - Actual field data

## Message Layout

```
+----------------------+---------------------------+
| Field Count (u16)    | Field Table (N entries)   |
+----------------------+---------------------------+
| Offset-to-Field-0    | Offset-to-Field-1 ...     |
+----------------------+---------------------------+
| Payload Section                                  |
+--------------------------------------------------+
```

### Message Header

- **Field Count** (2 bytes): Number of fields in the message (0-65535)
- **Field Table** (5 bytes × field_count): Array of field entries

### Field Table Entry

Each field entry is 5 bytes:

```
+------------------+------------------+
| Type ID (u8)     | Offset (u32)     |
+------------------+------------------+
```

- **Type ID** (1 byte): Primitive type identifier
- **Offset** (4 bytes): Absolute offset from message start to field data

## Type IDs

| Type ID | Type      | Description                     |
|---------|-----------|---------------------------------|
| 0       | u8        | 8-bit unsigned integer         |
| 1       | u16       | 16-bit unsigned integer        |
| 2       | u32       | 32-bit unsigned integer        |
| 3       | u64       | 64-bit unsigned integer        |
| 4       | i8        | 8-bit signed integer           |
| 5       | i16       | 16-bit signed integer          |
| 6       | i32       | 32-bit signed integer          |
| 7       | i64       | 64-bit signed integer          |
| 8       | f32       | 32-bit floating point          |
| 9       | f64       | 64-bit floating point          |
| 10      | bool      | Boolean value                  |
| 11      | string    | UTF-8 string                   |
| 12      | bytes     | Byte array                     |
| 13      | message   | Nested message                 |
| 14      | vector    | Vector of values               |

## Payload Encoding

### Scalar Types

All scalar types use little-endian byte order:

- **u8, i8, bool**: 1 byte
- **u16, i16**: 2 bytes
- **u32, i32, f32**: 4 bytes  
- **u64, i64, f64**: 8 bytes

### String and Bytes

```
+------------------+------------------+
| Length (u32)     | Bytes...         |
+------------------+------------------+
```

- **Length** (4 bytes): Number of bytes in the string/byte array
- **Bytes** (variable): UTF-8 string bytes or raw bytes

### Nested Message

```
+------------------+------------------+
| Length (u32)     | Message Data...  |
+------------------+------------------+
```

- **Length** (4 bytes): Size of the nested message in bytes
- **Message Data** (variable): Complete nested message data

### Vector

```
+------------------+------------------+------------------+
| Count (u32)      | Element 0        | Element 1 ...    |
+------------------+------------------+------------------+
```

- **Count** (4 bytes): Number of elements in the vector
- **Elements** (variable × count): Array of element data

## Alignment and Padding

ZeroProto does **not** use padding or alignment. All fields are packed tightly together to minimize space usage.

## Endianness

All multi-byte values use **little-endian** byte order for consistency across platforms.

## Example

Consider this schema:

```
message User {
    id: u64;
    name: string;
    age: u8;
}
```

With these values:
- id = 12345
- name = "Alice"  
- age = 30

The binary layout would be:

```
0000: 03 00                    // Field count = 3
0002: 03 00 00 00 10          // Field 0: type=3 (u64), offset=16
0007: 0B 00 00 00 18          // Field 1: type=11 (string), offset=24
000C: 00 00 00 00 1D          // Field 2: type=0 (u8), offset=29
0011: 39 30 00 00 00 00 00 00 // id = 12345 (u64)
0019: 05 00 00 00             // string length = 5
001D: 41 6C 69 63 65          // "Alice"
0022: 1E                       // age = 30 (u8)
```

## Validation Rules

### Message Validation

1. Field count must be ≤ 65535
2. All field offsets must be within the message bounds
3. Field offsets must be strictly increasing
4. Field table size must match field count

### Field Validation

1. String length must match actual UTF-8 byte count
2. Vector count must match actual element count
3. Nested message length must match embedded message size
4. All offsets must be properly aligned for their type

### Type Safety

1. Type ID must be valid (0-14)
2. Field data must match expected type
3. Nested vectors are not allowed

## Performance Considerations

### Zero-Copy Deserialization

- String and bytes fields borrow directly from the input buffer
- No memory allocations during deserialization
- Lifetime management ensures buffer validity

### Cache Efficiency

- Field table is small and typically cached
- Sequential access patterns for field data
- Minimal pointer indirection

### Size Optimization

- No padding or alignment bytes
- Compact field table (5 bytes per field)
- Variable-length encoding for strings and vectors

## Compatibility

### Version Compatibility

- Adding new fields is forward and backward compatible
- Removing fields breaks compatibility
- Changing field types breaks compatibility

### Platform Compatibility

- Little-endian byte order ensures consistency
- Fixed-size integer types prevent ambiguity
- UTF-8 encoding for strings ensures text compatibility

## Security Considerations

### Buffer Safety

- All offsets are bounds-checked
- String lengths are validated
- Vector counts are validated

### Memory Safety

- Zero-copy design prevents buffer overflows
- Lifetime tracking prevents use-after-free
- Type system prevents invalid field access

## Implementation Notes

### Reader Implementation

1. Parse field count and create field table
2. Bounds-check all field offsets
3. Provide typed access methods for each field type
4. Handle vector and nested message traversal

### Builder Implementation

1. Track field order and offsets
2. Serialize field data to buffer
3. Build field table during finalization
4. Handle variable-length field sizing

### Error Handling

- Return descriptive errors for invalid data
- Graceful handling of malformed messages
- Detailed error reporting for debugging
