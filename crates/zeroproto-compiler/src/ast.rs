//! Abstract Syntax Tree for ZeroProto schema language

use std::collections::HashMap;

/// A complete ZeroProto schema file
#[derive(Debug, Clone)]
pub struct Schema {
    pub items: Vec<SchemaItem>,
}

/// Items that can appear in a schema
#[derive(Debug, Clone)]
pub enum SchemaItem {
    Message(Message),
    Enum(Enum),
}

/// A message definition
#[derive(Debug, Clone)]
pub struct Message {
    pub name: String,
    pub fields: Vec<Field>,
}

/// A field in a message
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

/// The type of a field
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Scalar(ScalarType),
    UserDefined(String),
    Vector(Box<FieldType>),
}

/// Scalar types supported by ZeroProto
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScalarType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    String,
    Bytes,
}

/// An enum definition
#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

/// A variant in an enum
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,
}

impl Schema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add an item to the schema
    pub fn add_item(&mut self, item: SchemaItem) {
        self.items.push(item);
    }

    /// Get all message definitions
    pub fn messages(&self) -> impl Iterator<Item = &Message> {
        self.items.iter().filter_map(|item| match item {
            SchemaItem::Message(msg) => Some(msg),
            _ => None,
        })
    }

    /// Get all enum definitions
    pub fn enums(&self) -> impl Iterator<Item = &Enum> {
        self.items.iter().filter_map(|item| match item {
            SchemaItem::Enum(en) => Some(en),
            _ => None,
        })
    }

    /// Find a message by name
    pub fn find_message(&self, name: &str) -> Option<&Message> {
        self.messages().find(|msg| msg.name == name)
    }

    /// Find an enum by name
    pub fn find_enum(&self, name: &str) -> Option<&Enum> {
        self.enums().find(|en| en.name == name)
    }

    /// Validate the schema for basic consistency
    pub fn validate_basic(&self) -> Result<(), String> {
        let mut names = HashMap::new();

        for item in &self.items {
            let name = match item {
                SchemaItem::Message(msg) => &msg.name,
                SchemaItem::Enum(en) => &en.name,
            };

            if names.contains_key(name) {
                return Err(format!("Duplicate name '{}' found", name));
            }
            names.insert(name.clone(), ());
        }

        // Validate field types
        for message in self.messages() {
            for field in &message.fields {
                self.validate_field_type(&field.field_type)?;
            }
        }

        Ok(())
    }

    /// Validate a field type
    fn validate_field_type(&self, field_type: &FieldType) -> Result<(), String> {
        match field_type {
            FieldType::Scalar(_) => Ok(()),
            FieldType::UserDefined(name) => {
                if self.find_message(name).is_none() && self.find_enum(name).is_none() {
                    Err(format!("Unknown type '{}'", name))
                } else {
                    Ok(())
                }
            }
            FieldType::Vector(inner) => self.validate_field_type(inner),
        }
    }
}

impl Message {
    /// Create a new message
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
        }
    }

    /// Add a field to the message
    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    /// Find a field by name
    pub fn find_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|field| field.name == name)
    }
}

impl Field {
    /// Create a new field
    pub fn new(name: String, field_type: FieldType) -> Self {
        Self { name, field_type }
    }
}

impl Enum {
    /// Create a new enum
    pub fn new(name: String) -> Self {
        Self {
            name,
            variants: Vec::new(),
        }
    }

    /// Add a variant to the enum
    pub fn add_variant(&mut self, variant: EnumVariant) {
        self.variants.push(variant);
    }

    /// Find a variant by name
    pub fn find_variant(&self, name: &str) -> Option<&EnumVariant> {
        self.variants.iter().find(|variant| variant.name == name)
    }
}

impl EnumVariant {
    /// Create a new variant
    pub fn new(name: String) -> Self {
        Self { name, value: None }
    }

    /// Create a new variant with an explicit value
    pub fn with_value(name: String, value: i64) -> Self {
        Self { name, value: Some(value) }
    }
}

impl ScalarType {
    /// Get the Rust type name for this scalar
    pub fn rust_type(&self) -> &'static str {
        match self {
            ScalarType::U8 => "u8",
            ScalarType::U16 => "u16",
            ScalarType::U32 => "u32",
            ScalarType::U64 => "u64",
            ScalarType::I8 => "i8",
            ScalarType::I16 => "i16",
            ScalarType::I32 => "i32",
            ScalarType::I64 => "i64",
            ScalarType::F32 => "f32",
            ScalarType::F64 => "f64",
            ScalarType::Bool => "bool",
            ScalarType::String => "&'a str",
            ScalarType::Bytes => "&'a [u8]",
        }
    }

    /// Get the primitive type ID for this scalar
    pub fn primitive_type_id(&self) -> u8 {
        use crate::primitives::PrimitiveType;
        match self {
            ScalarType::U8 => PrimitiveType::U8 as u8,
            ScalarType::U16 => PrimitiveType::U16 as u8,
            ScalarType::U32 => PrimitiveType::U32 as u8,
            ScalarType::U64 => PrimitiveType::U64 as u8,
            ScalarType::I8 => PrimitiveType::I8 as u8,
            ScalarType::I16 => PrimitiveType::I16 as u8,
            ScalarType::I32 => PrimitiveType::I32 as u8,
            ScalarType::I64 => PrimitiveType::I64 as u8,
            ScalarType::F32 => PrimitiveType::F32 as u8,
            ScalarType::F64 => PrimitiveType::F64 as u8,
            ScalarType::Bool => PrimitiveType::Bool as u8,
            ScalarType::String => PrimitiveType::String as u8,
            ScalarType::Bytes => PrimitiveType::Bytes as u8,
        }
    }
}
