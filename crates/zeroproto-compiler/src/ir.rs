//! Intermediate Representation for ZeroProto code generation

use crate::ast::*;
use std::collections::HashMap;

/// Intermediate representation of a ZeroProto schema
#[derive(Debug, Clone)]
pub struct IrSchema {
    pub messages: Vec<IrMessage>,
    pub enums: Vec<IrEnum>,
}

/// Intermediate representation of a message
#[derive(Debug, Clone)]
pub struct IrMessage {
    pub name: String,
    pub rust_name: String,
    pub fields: Vec<IrField>,
    pub reader_name: String,
    pub builder_name: String,
}

/// Intermediate representation of a field
#[derive(Debug, Clone)]
pub struct IrField {
    pub name: String,
    pub rust_name: String,
    pub field_type: IrFieldType,
    pub index: u16,
    pub offset_constant: String,
    /// Whether this field is optional
    pub optional: bool,
    /// Default value for this field (as Rust code string)
    pub default_value: Option<String>,
}

/// Intermediate representation of a field type
#[derive(Debug, Clone)]
pub enum IrFieldType {
    Scalar {
        scalar_type: ScalarType,
        rust_type: String,
        primitive_id: u8,
    },
    UserDefined {
        type_name: String,
        rust_type: String,
        is_message: bool,
    },
    Vector {
        element_type: Box<IrFieldType>,
        rust_type: String,
        reader_type: String,
    },
}

/// Intermediate representation of an enum
#[derive(Debug, Clone)]
pub struct IrEnum {
    pub name: String,
    pub rust_name: String,
    pub variants: Vec<IrEnumVariant>,
}

/// Intermediate representation of an enum variant
#[derive(Debug, Clone)]
pub struct IrEnumVariant {
    pub name: String,
    pub rust_name: String,
    pub value: i64,
}

/// Convert AST to IR
pub fn lower_ast(schema: &Schema) -> IrSchema {
    let mut ir = IrSchema {
        messages: Vec::new(),
        enums: Vec::new(),
    };

    // First, convert all enums (needed for message field types)
    for enum_def in schema.enums() {
        ir.enums.push(lower_enum(enum_def));
    }

    // Then convert all messages
    for message in schema.messages() {
        ir.messages.push(lower_message(message, &ir.enums));
    }

    ir
}

/// Convert an AST enum to IR
fn lower_enum(enum_def: &Enum) -> IrEnum {
    let rust_name = to_pascal_case(&enum_def.name);
    let variants: Vec<_> = enum_def
        .variants
        .iter()
        .enumerate()
        .map(|(i, variant)| IrEnumVariant {
            name: variant.name.clone(),
            rust_name: to_pascal_case(&variant.name),
            value: variant.value.unwrap_or(i as i64),
        })
        .collect();

    IrEnum {
        name: enum_def.name.clone(),
        rust_name,
        variants,
    }
}

/// Convert an AST message to IR
fn lower_message(message: &Message, enums: &[IrEnum]) -> IrMessage {
    let rust_name = to_pascal_case(&message.name);
    let reader_name = format!("{}Reader", rust_name);
    let builder_name = format!("{}Builder", rust_name);

    let fields: Vec<_> = message
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let field_type = lower_field_type(&field.field_type, enums);
            let default_value = field.default_value.as_ref().map(|dv| {
                lower_default_value(dv, &field.field_type)
            });
            IrField {
                name: field.name.clone(),
                rust_name: to_snake_case(&field.name),
                field_type,
                index: i as u16,
                offset_constant: format!("FIELD_{}_OFFSET", i),
                optional: field.optional,
                default_value,
            }
        })
        .collect();

    IrMessage {
        name: message.name.clone(),
        rust_name,
        fields,
        reader_name,
        builder_name,
    }
}

/// Convert a default value to Rust code
fn lower_default_value(default: &crate::ast::DefaultValue, _field_type: &FieldType) -> String {
    use crate::ast::DefaultValue;
    
    match default {
        DefaultValue::Integer(val) => val.to_string(),
        DefaultValue::Float(val) => {
            // Ensure float literals have a decimal point
            let s = val.to_string();
            if s.contains('.') { s } else { format!("{}.0", s) }
        }
        DefaultValue::Bool(val) => val.to_string(),
        DefaultValue::String(val) => format!("\"{}\"", val),
    }
}

/// Convert an AST field type to IR
fn lower_field_type(field_type: &FieldType, enums: &[IrEnum]) -> IrFieldType {
    match field_type {
        FieldType::Scalar(scalar_type) => IrFieldType::Scalar {
            scalar_type: scalar_type.clone(),
            rust_type: scalar_type.rust_type().to_string(),
            primitive_id: scalar_type.primitive_type_id(),
        },
        FieldType::UserDefined(type_name) => {
            let rust_type = to_pascal_case(type_name);
            let is_message = !enums.iter().any(|en| en.name == *type_name);
            
            IrFieldType::UserDefined {
                type_name: type_name.clone(),
                rust_type,
                is_message,
            }
        }
        FieldType::Vector(inner) => {
            let element_type = Box::new(lower_field_type(inner, enums));
            let rust_type = match element_type.as_ref() {
                IrFieldType::Scalar { rust_type, .. } => {
                    format!("VectorReader<'a, {}>", rust_type)
                }
                IrFieldType::UserDefined { rust_type, is_message, .. } => {
                    if *is_message {
                        format!("VectorReader<'a, {}Reader<'a>>", rust_type)
                    } else {
                        format!("VectorReader<'a, {}>", rust_type)
                    }
                }
                IrFieldType::Vector { .. } => panic!("Nested vectors should have been caught by validator"),
            };

            IrFieldType::Vector {
                element_type,
                rust_type,
                reader_type: format!("VectorReader<'a, _>"),
            }
        }
    }
}

/// Convert to PascalCase
fn to_pascal_case(input: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in input.chars() {
        if ch == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert to snake_case
fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_char_was_upper = false;

    for (i, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_char_was_upper {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_char_was_upper = true;
        } else {
            result.push(ch);
            prev_char_was_upper = false;
        }
    }

    // Handle edge cases
    if result.ends_with('_') {
        result.pop();
    }

    result
}

/// Utility functions for working with IR
pub struct IrUtils;

impl IrUtils {
    /// Get all dependencies of a message
    pub fn get_message_dependencies<'a>(
        message: &'a IrMessage,
        schema: &'a IrSchema,
    ) -> Vec<&'a str> {
        let mut dependencies = Vec::new();
        
        for field in &message.fields {
            Self::collect_field_dependencies(&field.field_type, schema, &mut dependencies);
        }
        
        dependencies.sort();
        dependencies.dedup();
        dependencies
    }

    /// Collect dependencies from a field type
    fn collect_field_dependencies<'a>(
        field_type: &'a IrFieldType,
        schema: &'a IrSchema,
        dependencies: &mut Vec<&'a str>,
    ) {
        match field_type {
            IrFieldType::Scalar { .. } => {}
            IrFieldType::UserDefined { type_name, .. } => {
                dependencies.push(type_name.as_str());
            }
            IrFieldType::Vector { element_type, .. } => {
                Self::collect_field_dependencies(element_type, schema, dependencies);
            }
        }
    }

    /// Calculate the field table size for a message
    pub fn field_table_size(message: &IrMessage) -> usize {
        message.fields.len() * 5 // type_id (1) + offset (4)
    }

    /// Generate field offset constants
    pub fn generate_field_offsets(message: &IrMessage) -> Vec<(String, usize)> {
        let mut offsets = Vec::new();
        let mut current_offset = 2 + Self::field_table_size(message); // Skip header and field table

        for field in &message.fields {
            offsets.push((field.offset_constant.clone(), current_offset));
            current_offset += Self::field_size(&field.field_type);
        }

        offsets
    }

    /// Calculate the size of a field
    pub fn field_size(field_type: &IrFieldType) -> usize {
        match field_type {
            IrFieldType::Scalar { scalar_type, .. } => scalar_type.size(),
            IrFieldType::UserDefined { is_message, .. } => {
                if *is_message {
                    4 // Length prefix for nested message
                } else {
                    4 // Length prefix for enum
                }
            }
            IrFieldType::Vector { element_type, .. } => {
                4 + Self::field_size(element_type) // count + one element
            }
        }
    }

    /// Check if a type requires a lifetime parameter
    pub fn requires_lifetime(field_type: &IrFieldType) -> bool {
        match field_type {
            IrFieldType::Scalar { scalar_type, .. } => {
                matches!(
                    scalar_type,
                    ScalarType::String | ScalarType::Bytes
                )
            }
            IrFieldType::UserDefined { is_message, .. } => *is_message,
            IrFieldType::Vector { element_type, .. } => Self::requires_lifetime(element_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("id"), "Id");
        assert_eq!(to_pascal_case("profile"), "Profile");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("UserName"), "user_name");
        assert_eq!(to_snake_case("ID"), "id");
        assert_eq!(to_snake_case("Profile"), "profile");
    }

    #[test]
    fn test_ir_lowering() {
        let mut schema = Schema::new();
        let mut message = Message::new("user".to_string());
        message.add_field(Field::new("id".to_string(), FieldType::Scalar(ScalarType::U64)));
        message.add_field(Field::new("name".to_string(), FieldType::Scalar(ScalarType::String)));
        schema.add_item(SchemaItem::Message(message));

        let ir = lower_ast(&schema);
        assert_eq!(ir.messages.len(), 1);
        assert_eq!(ir.messages[0].rust_name, "User");
        assert_eq!(ir.messages[0].reader_name, "UserReader");
        assert_eq!(ir.messages[0].builder_name, "UserBuilder");
        assert_eq!(ir.messages[0].fields.len(), 2);
    }
}
