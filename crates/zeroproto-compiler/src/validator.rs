//! Validator for ZeroProto schema AST

use crate::ast::*;
use crate::Result;
use std::collections::HashMap;

/// Validate a schema AST
pub fn validate(schema: &Schema) -> Result<()> {
    let mut validator = SchemaValidator::new();
    validator.validate(schema)
}

/// Schema validation context
struct SchemaValidator {
    type_names: HashMap<String, TypeKind>,
    current_message: Option<String>,
}

/// Kinds of types that can be defined
#[derive(Debug, Clone, Copy)]
enum TypeKind {
    Message,
    Enum,
}

impl SchemaValidator {
    /// Create a new validator
    fn new() -> Self {
        Self {
            type_names: HashMap::new(),
            current_message: None,
        }
    }

    /// Validate the entire schema
    fn validate(&mut self, schema: &Schema) -> Result<()> {
        // First pass: collect all type names
        self.collect_type_names(schema)?;

        // Second pass: validate all items
        for item in &schema.items {
            match item {
                SchemaItem::Message(msg) => self.validate_message(msg)?,
                SchemaItem::Enum(en) => self.validate_enum(en)?,
            }
        }

        Ok(())
    }

    /// Collect all type names from the schema
    fn collect_type_names(&mut self, schema: &Schema) -> Result<()> {
        for item in &schema.items {
            let name = match item {
                SchemaItem::Message(msg) => &msg.name,
                SchemaItem::Enum(en) => &en.name,
            };

            if self.type_names.contains_key(name) {
                return Err(crate::CompilerError::Validation(format!(
                    "Duplicate type name '{}' found",
                    name
                )));
            }

            let kind = match item {
                SchemaItem::Message(_) => TypeKind::Message,
                SchemaItem::Enum(_) => TypeKind::Enum,
            };

            self.type_names.insert(name.clone(), kind);
        }

        Ok(())
    }

    /// Validate a message definition
    fn validate_message(&mut self, message: &Message) -> Result<()> {
        self.current_message = Some(message.name.clone());

        // Check for reserved field names
        let reserved_names = ["type", "id", "data", "buffer"];
        for field in &message.fields {
            if reserved_names.contains(&field.name.as_str()) {
                return Err(crate::CompilerError::Validation(format!(
                    "Field name '{}' is reserved in message '{}'",
                    field.name, message.name
                )));
            }
        }

        // Validate field types
        for field in &message.fields {
            self.validate_field_type(&field.field_type)?;
        }

        // Check for duplicate field names
        let mut field_names = HashMap::new();
        for field in &message.fields {
            if field_names.contains_key(&field.name) {
                return Err(crate::CompilerError::Validation(format!(
                    "Duplicate field name '{}' in message '{}'",
                    field.name, message.name
                )));
            }
            field_names.insert(field.name.clone(), ());
        }

        self.current_message = None;
        Ok(())
    }

    /// Validate a field type
    fn validate_field_type(&self, field_type: &FieldType) -> Result<()> {
        match field_type {
            FieldType::Scalar(_) => Ok(()),
            FieldType::UserDefined(name) => {
                if !self.type_names.contains_key(name) {
                    return Err(crate::CompilerError::Validation(format!(
                        "Unknown type '{}' used in field",
                        name
                    )));
                }
                Ok(())
            }
            FieldType::Vector(inner) => {
                // Validate the inner type
                self.validate_field_type(inner)?;
                
                // Check for nested vectors (not allowed)
                if matches!(inner.as_ref(), FieldType::Vector(_)) {
                    return Err(crate::CompilerError::Validation(
                        "Nested vectors are not allowed".to_string()
                    ));
                }
                
                Ok(())
            }
        }
    }

    /// Validate an enum definition
    fn validate_enum(&self, enum_def: &Enum) -> Result<()> {
        // Check for duplicate variant names
        let mut variant_names = HashMap::new();
        for variant in &enum_def.variants {
            if variant_names.contains_key(&variant.name) {
                return Err(crate::CompilerError::Validation(format!(
                    "Duplicate variant name '{}' in enum '{}'",
                    variant.name, enum_def.name
                )));
            }
            variant_names.insert(variant.name.clone(), ());
        }

        // Validate variant values
        let mut used_values = HashMap::new();
        for (i, variant) in enum_def.variants.iter().enumerate() {
            let value = if let Some(explicit_value) = variant.value {
                explicit_value
            } else {
                // Auto-assign sequential values starting from 0
                i as i64
            };

            if used_values.contains_key(&value) {
                return Err(crate::CompilerError::Validation(format!(
                    "Duplicate enum value {} in enum '{}'",
                    value, enum_def.name
                )));
            }
            used_values.insert(value, ());
        }

        // Check for reserved enum names
        let reserved_names = ["Result", "Option", "Status"];
        if reserved_names.contains(&enum_def.name.as_str()) {
            return Err(crate::CompilerError::Validation(format!(
                "Enum name '{}' is reserved",
                enum_def.name
            )));
        }

        Ok(())
    }
}

/// Additional validation utilities
pub struct ValidationUtils;

impl ValidationUtils {
    /// Check if a field type is valid for zero-copy
    pub fn is_zero_copy_compatible(field_type: &FieldType) -> bool {
        match field_type {
            FieldType::Scalar(scalar) => matches!(
                scalar,
                ScalarType::U8 | ScalarType::U16 | ScalarType::U32 | ScalarType::U64 |
                ScalarType::I8 | ScalarType::I16 | ScalarType::I32 | ScalarType::I64 |
                ScalarType::F32 | ScalarType::F64 | ScalarType::Bool |
                ScalarType::String | ScalarType::Bytes
            ),
            FieldType::UserDefined(_) => true, // Will be validated elsewhere
            FieldType::Vector(inner) => Self::is_zero_copy_compatible(inner),
        }
    }

    /// Calculate the maximum possible size of a field
    pub fn max_field_size(field_type: &FieldType) -> Option<usize> {
        match field_type {
            FieldType::Scalar(scalar) => Some(scalar.size()),
            FieldType::UserDefined(_) => None, // Unknown size for user-defined types
            FieldType::Vector(inner) => {
                let inner_size = Self::max_field_size(inner)?;
                Some(4 + inner_size) // count + one element
            }
        }
    }
}

impl ScalarType {
    /// Get the size of this scalar type in bytes
    pub fn size(&self) -> usize {
        match self {
            ScalarType::U8 | ScalarType::I8 | ScalarType::Bool => 1,
            ScalarType::U16 | ScalarType::I16 => 2,
            ScalarType::U32 | ScalarType::I32 | ScalarType::F32 => 4,
            ScalarType::U64 | ScalarType::I64 | ScalarType::F64 => 8,
            ScalarType::String | ScalarType::Bytes => 4, // Length prefix only
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_message() {
        let mut schema = Schema::new();
        let mut message = Message::new("User".to_string());
        message.add_field(Field::new("user_id".to_string(), FieldType::Scalar(ScalarType::U64)));
        message.add_field(Field::new("name".to_string(), FieldType::Scalar(ScalarType::String)));
        schema.add_item(SchemaItem::Message(message));

        assert!(validate(&schema).is_ok());
    }

    #[test]
    fn test_duplicate_type_names() {
        let mut schema = Schema::new();
        schema.add_item(SchemaItem::Message(Message::new("User".to_string())));
        schema.add_item(SchemaItem::Message(Message::new("User".to_string())));

        assert!(validate(&schema).is_err());
    }

    #[test]
    fn test_unknown_field_type() {
        let mut schema = Schema::new();
        let mut message = Message::new("User".to_string());
        message.add_field(Field::new("profile".to_string(), FieldType::UserDefined("Profile".to_string())));
        schema.add_item(SchemaItem::Message(message));

        assert!(validate(&schema).is_err());
    }

    #[test]
    fn test_nested_vector() {
        let mut schema = Schema::new();
        let mut message = Message::new("User".to_string());
        let nested_vector = FieldType::Vector(Box::new(FieldType::Vector(Box::new(
            FieldType::Scalar(ScalarType::U64)
        ))));
        message.add_field(Field::new("bad_field".to_string(), nested_vector));
        schema.add_item(SchemaItem::Message(message));

        assert!(validate(&schema).is_err());
    }
}
