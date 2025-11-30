//! Primitive type definitions for ZeroProto

/// Primitive type IDs
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    I8 = 4,
    I16 = 5,
    I32 = 6,
    I64 = 7,
    F32 = 8,
    F64 = 9,
    Bool = 10,
    String = 11,
    Bytes = 12,
}
