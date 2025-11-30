//! Parser for ZeroProto schema language

use crate::ast::*;
use crate::Result;

// Import the specific types we need
use crate::ast::{FieldType, ScalarType};

/// Parse a ZeroProto schema string into an AST
pub fn parse(input: &str) -> Result<Schema> {
    let mut parser = SchemaParser::new();
    parser.parse(input)
}

/// Simple hand-written parser for ZeroProto schema language
struct SchemaParser {
    tokens: Vec<Token>,
    position: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Identifier(String),
    Integer(i64),
    String(String),
    Message,
    Enum,
    Colon,
    Semicolon,
    Comma,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Equals,
}

impl SchemaParser {
    fn new() -> Self {
        Self {
            tokens: Vec::new(),
            position: 0,
        }
    }

    fn tokenize(&mut self, input: &str) -> Result<()> {
        self.tokens.clear();
        self.position = 0;
        
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;
        
        while i < chars.len() {
            match chars[i] {
                ' ' | '\t' | '\n' | '\r' => i += 1,
                '/' if i + 1 < chars.len() && chars[i + 1] == '/' => {
                    // Line comment
                    while i < chars.len() && chars[i] != '\n' {
                        i += 1;
                    }
                }
                '/' if i + 1 < chars.len() && chars[i + 1] == '*' => {
                    // Block comment
                    i += 2;
                    while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                        i += 1;
                    }
                    i += 2;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let start = i;
                    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                        i += 1;
                    }
                    let ident: String = chars[start..i].iter().collect();
                    
                    let token = match ident.as_str() {
                        "message" => Token::Message,
                        "enum" => Token::Enum,
                        "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" |
                        "f32" | "f64" | "bool" | "string" | "bytes" => Token::Identifier(ident),
                        _ => Token::Identifier(ident),
                    };
                    self.tokens.push(token);
                }
                '0'..='9' => {
                    let start = i;
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let num_str: String = chars[start..i].iter().collect();
                    let num = num_str.parse().expect("valid integer");
                    self.tokens.push(Token::Integer(num));
                }
                ':' => {
                    self.tokens.push(Token::Colon);
                    i += 1;
                }
                ';' => {
                    self.tokens.push(Token::Semicolon);
                    i += 1;
                }
                '{' => {
                    self.tokens.push(Token::LeftBrace);
                    i += 1;
                }
                '}' => {
                    self.tokens.push(Token::RightBrace);
                    i += 1;
                }
                '[' => {
                    self.tokens.push(Token::LeftBracket);
                    i += 1;
                }
                ']' => {
                    self.tokens.push(Token::RightBracket);
                    i += 1;
                }
                '=' => {
                    self.tokens.push(Token::Equals);
                    i += 1;
                }
                ',' => {
                    self.tokens.push(Token::Comma);
                    i += 1;
                }
                _ => {
                    return Err(crate::CompilerError::Parse(format!("Unexpected character: {}", chars[i])));
                }
            }
        }
        Ok(())
    }

    fn parse(&mut self, input: &str) -> Result<Schema> {
        self.tokenize(input)?;
        
        let mut items = Vec::new();
        
        while !self.at_end() {
            if self.peek() == Token::Message {
                items.push(self.parse_message()?);
            } else if self.peek() == Token::Enum {
                items.push(self.parse_enum()?);
            } else {
                return Err(crate::CompilerError::Parse("Expected message or enum".to_string()));
            }
        }
        
        Ok(Schema { items })
    }

    fn parse_message(&mut self) -> Result<SchemaItem> {
        self.consume(Token::Message)?;
        let name = self.consume_identifier()?;
        self.consume(Token::LeftBrace)?;
        
        let fields = self.parse_message_fields()?;
        
        self.consume(Token::RightBrace)?;
        Ok(SchemaItem::Message(Message { name, fields }))
    }

    fn parse_enum(&mut self) -> Result<SchemaItem> {
        self.consume(Token::Enum)?;
        let name = self.consume_identifier()?;
        self.consume(Token::LeftBrace)?;
        
        let mut variants = Vec::new();
        while !self.at_end() && self.peek() != Token::RightBrace {
            variants.push(self.parse_enum_variant()?);
        }
        
        self.consume(Token::RightBrace)?;
        Ok(SchemaItem::Enum(Enum { name, variants }))
    }

    fn parse_field(&mut self) -> Result<Field> {
        let name = self.consume_identifier()?;
        self.consume(Token::Colon)?;
        let field_type = self.parse_type()?;
        self.consume(Token::Semicolon)?;
        Ok(Field { name, field_type })
    }

    fn parse_message_fields(&mut self) -> Result<Vec<Field>> {
        let mut fields = Vec::new();
        while !self.at_end() && self.peek() != Token::RightBrace {
            fields.push(self.parse_field()?);
            // Optional comma between fields
            if self.peek() == Token::Comma {
                self.consume(Token::Comma)?;
            }
        }
        Ok(fields)
    }

    fn parse_enum_variant(&mut self) -> Result<EnumVariant> {
        let name = self.consume_identifier()?;
        self.consume(Token::Equals)?;
        let value = self.consume_integer()?;
        self.consume(Token::Semicolon)?;
        Ok(EnumVariant { name, value: Some(value) })
    }

    fn parse_type(&mut self) -> Result<FieldType> {
        if self.peek() == Token::LeftBracket {
            self.consume(Token::LeftBracket)?;
            let inner = self.parse_type()?;
            self.consume(Token::RightBracket)?;
            Ok(FieldType::Vector(Box::new(inner)))
        } else {
            let ident = self.consume_identifier()?;
            let scalar_type = match ident.as_str() {
                "u8" => ScalarType::U8,
                "u16" => ScalarType::U16,
                "u32" => ScalarType::U32,
                "u64" => ScalarType::U64,
                "i8" => ScalarType::I8,
                "i16" => ScalarType::I16,
                "i32" => ScalarType::I32,
                "i64" => ScalarType::I64,
                "f32" => ScalarType::F32,
                "f64" => ScalarType::F64,
                "bool" => ScalarType::Bool,
                "string" => ScalarType::String,
                "bytes" => ScalarType::Bytes,
                _ => return Ok(FieldType::UserDefined(ident)),
            };
            Ok(FieldType::Scalar(scalar_type))
        }
    }

    fn at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn peek(&self) -> Token {
        if self.at_end() {
            panic!("Unexpected end of input");
        }
        self.tokens[self.position].clone()
    }

    fn consume(&mut self, expected: Token) -> Result<()> {
        if self.at_end() {
            return Err(crate::CompilerError::Parse(format!("Expected {:?}, found end of input", expected)));
        }
        
        let token = self.tokens[self.position].clone();
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
            self.position += 1;
            Ok(())
        } else {
            Err(crate::CompilerError::Parse(format!("Expected {:?}, found {:?}", expected, token)))
        }
    }

    fn consume_identifier(&mut self) -> Result<String> {
        if self.at_end() {
            return Err(crate::CompilerError::Parse("Expected identifier, found end of input".to_string()));
        }
        
        match &self.tokens[self.position] {
            Token::Identifier(name) => {
                self.position += 1;
                Ok(name.clone())
            }
            _ => Err(crate::CompilerError::Parse("Expected identifier".to_string())),
        }
    }

    fn consume_integer(&mut self) -> Result<i64> {
        if self.at_end() {
            return Err(crate::CompilerError::Parse("Expected integer, found end of input".to_string()));
        }
        
        match &self.tokens[self.position] {
            Token::Integer(value) => {
                self.position += 1;
                Ok(*value)
            }
            _ => Err(crate::CompilerError::Parse("Expected integer".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_message() {
        let input = r#"
            message User {
                id: u64;
                name: string;
            }
        "#;
        
        let schema = parse(input).unwrap();
        assert_eq!(schema.items.len(), 1);
        
        if let SchemaItem::Message(msg) = &schema.items[0] {
            assert_eq!(msg.name, "User");
            assert_eq!(msg.fields.len(), 2);
            assert_eq!(msg.fields[0].name, "id");
            assert_eq!(msg.fields[1].name, "name");
        } else {
            panic!("Expected message");
        }
    }

    #[test]
    fn test_vector_field() {
        let input = r#"
            message User {
                friends: [u64];
            }
        "#;
        
        let schema = parse(input).unwrap();
        
        if let SchemaItem::Message(msg) = &schema.items[0] {
            assert_eq!(msg.name, "User");
            assert_eq!(msg.fields.len(), 1);
            
            let field = &msg.fields[0];
            assert_eq!(field.name, "friends");
            
            if let FieldType::Vector(inner) = &field.field_type {
                if let FieldType::Scalar(ScalarType::U64) = **inner {
                    // Correct
                } else {
                    panic!("Expected vector of u64");
                }
            } else {
                panic!("Expected vector type");
            }
        } else {
            panic!("Expected message");
        }
    }
}
