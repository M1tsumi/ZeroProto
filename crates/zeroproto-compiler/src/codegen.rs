//! Code generation for ZeroProto schemas

use crate::ir::*;
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

/// Generate Rust code from IR
pub fn generate_rust_code(ir: &IrSchema) -> crate::Result<String> {
    let mut code = String::new();
    
    // Add imports
    code.push_str("use zeroproto::*;\n\n");
    
    // Generate enums first
    for enum_def in &ir.enums {
        code.push_str(&generate_enum(enum_def));
        code.push_str("\n\n");
    }
    
    // Generate messages
    for message in &ir.messages {
        code.push_str(&generate_message(ir, message));
        code.push_str("\n\n");
    }
    
    Ok(code)
}

/// Generate code for an enum
fn generate_enum(enum_def: &IrEnum) -> String {
    let name_ident = format_ident!("{}", enum_def.rust_name);
    let variants: Vec<_> = enum_def
        .variants
        .iter()
        .map(|variant| {
            let variant_name = format_ident!("{}", variant.rust_name);
            let value = Literal::i64_unsuffixed(variant.value);
            quote! { #variant_name = #value }
        })
        .collect();
    
    // Generate match arms manually
    let read_arms: Vec<_> = enum_def
        .variants
        .iter()
        .map(|variant| {
            let variant_name = format_ident!("{}", variant.rust_name);
            let value = Literal::i64_unsuffixed(variant.value);
            quote! { #value => Ok(#name_ident::#variant_name) }
        })
        .collect();
    
    let write_arms: Vec<_> = enum_def
        .variants
        .iter()
        .map(|variant| {
            let variant_name = format_ident!("{}", variant.rust_name);
            let value = Literal::i64_unsuffixed(variant.value);
            quote! { #name_ident::#variant_name => #value }
        })
        .collect();
    
    let code = quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #name_ident {
            #(#variants),*
        }
        
        impl #name_ident {
            /// Read this enum from a buffer
            pub fn read(buf: &[u8], offset: usize) -> zeroproto::Result<Self> {
                let value = Endian::Little.read_u64(buf, offset);
                match value {
                    #(
                        #read_arms,
                    )*
                    _ => Err(zeroproto::Error::InvalidFieldType),
                }
            }
            
            /// Write this enum to a buffer
            pub fn write(&self, buf: &mut [u8], offset: usize) -> zeroproto::Result<()> {
                let value = match self {
                    #(
                        #write_arms,
                    )*
                };
                Endian::Little.write_u64(value, buf, offset);
                Ok(())
            }
        }
    };
    
    code.to_string()
}

/// Generate code for a message
fn generate_message(ir: &IrSchema, message: &IrMessage) -> String {
    let reader_code = generate_reader(ir, message);
    let builder_code = generate_builder(ir, message);
    let constants_code = generate_constants(message);
    
    format!(
        "{}\n\n{}\n\n{}",
        constants_code, reader_code, builder_code
    )
}

/// Generate field offset constants
fn generate_constants(message: &IrMessage) -> String {
    let offsets = IrUtils::generate_field_offsets(message);
    let mut code = String::new();
    
    for (constant_name, offset) in offsets {
        code.push_str(&format!("pub const {}: usize = {};\n", constant_name, offset));
    }
    
    code
}

/// Generate reader code for a message
fn generate_reader(ir: &IrSchema, message: &IrMessage) -> String {
    let reader_name = format_ident!("{}", message.reader_name);
    let fields = &message.fields;
    
    let field_methods: Vec<_> = fields
        .iter()
        .map(|field| generate_reader_field_method(ir, field))
        .collect();
    
    let new_method = generate_reader_new_method(message);
    let from_slice_method = generate_reader_from_slice_method(message);
    
    let code = quote! {
        #new_method
        
        #from_slice_method
        
        impl<'a> #reader_name<'a> {
            #(#field_methods)*
        }
    };
    
    code.to_string()
}

/// Generate the new() method for readers
fn generate_reader_new_method(message: &IrMessage) -> TokenStream {
    let reader_name = format_ident!("{}", message.reader_name);
    
    quote! {
        /// Zero-copy reader for #reader_name messages
        #[derive(Debug)]
        pub struct #reader_name<'a> {
            reader: MessageReader<'a>,
        }
        
        impl<'a> #reader_name<'a> {
            /// Create a new reader from a message reader
            pub fn new(reader: MessageReader<'a>) -> Self {
                Self { reader }
            }
            
            /// Create a new reader from raw bytes
            pub fn from_bytes(data: &'a [u8]) -> zeroproto::Result<Self> {
                Ok(Self::new(MessageReader::new(data)?))
            }
        }
    }
}

/// Generate the from_slice() method for readers
fn generate_reader_from_slice_method(message: &IrMessage) -> TokenStream {
    let reader_name = format_ident!("{}", message.reader_name);
    
    quote! {
        impl<'a> #reader_name<'a> {
            /// Create a reader from a byte slice
            pub fn from_slice(data: &'a [u8]) -> zeroproto::Result<Self> {
                Self::from_bytes(data)
            }
        }
    }
}

/// Generate a reader field method
fn generate_reader_field_method(_ir: &IrSchema, field: &IrField) -> TokenStream {
    let method_name = format_ident!("{}", field.rust_name);
    let field_index = field.index;
    
    match &field.field_type {
        IrFieldType::Scalar { rust_type, .. } => {
            let return_type = syn::parse_str::<syn::Type>(rust_type).unwrap();
            quote! {
                /// Get the #method_name field
                pub fn #method_name(&self) -> zeroproto::Result<#return_type> {
                    self.reader.get_scalar(#field_index)
                }
            }
        }
        IrFieldType::UserDefined { rust_type, is_message, .. } => {
            if *is_message {
                let reader_type = format_ident!("{}Reader", rust_type);
                quote! {
                    /// Get the #method_name field
                    pub fn #method_name(&self) -> zeroproto::Result<#reader_type<'a>> {
                        let message_reader = self.reader.get_message(#field_index)?;
                        Ok(#reader_type::new(message_reader))
                    }
                }
            } else {
                let return_type = syn::parse_str::<syn::Type>(rust_type).unwrap();
                quote! {
                    /// Get the #method_name field
                    pub fn #method_name(&self) -> zeroproto::Result<#return_type> {
                        self.reader.get_scalar(#field_index)
                    }
                }
            }
        }
        IrFieldType::Vector { element_type, .. } => {
            generate_vector_reader_method(field, element_type)
        }
    }
}

/// Generate vector reader method
fn generate_vector_reader_method(field: &IrField, element_type: &IrFieldType) -> TokenStream {
    let method_name = format_ident!("{}", field.rust_name);
    let field_index = field.index;
    
    match element_type {
        IrFieldType::Scalar { rust_type, .. } => {
            let return_type = format!("VectorReader<'a, {}>", rust_type);
            let parsed_type = syn::parse_str::<syn::Type>(&return_type).unwrap();
            quote! {
                /// Get the #method_name field
                pub fn #method_name(&self) -> zeroproto::Result<#parsed_type> {
                    self.reader.get_vector(#field_index)
                }
            }
        }
        IrFieldType::UserDefined { rust_type, is_message, .. } => {
            if *is_message {
                let return_type = format!("VectorReader<'a, {}Reader<'a>>", rust_type);
                let parsed_type = syn::parse_str::<syn::Type>(&return_type).unwrap();
                quote! {
                    /// Get the #method_name field
                    pub fn #method_name(&self) -> zeroproto::Result<#parsed_type> {
                        self.reader.get_vector(#field_index)
                    }
                }
            } else {
                let return_type = format!("VectorReader<'a, {}>", rust_type);
                let parsed_type = syn::parse_str::<syn::Type>(&return_type).unwrap();
                quote! {
                    /// Get the #method_name field
                    pub fn #method_name(&self) -> zeroproto::Result<#parsed_type> {
                        self.reader.get_vector(#field_index)
                    }
                }
            }
        }
        IrFieldType::Vector { .. } => panic!("Nested vectors should have been caught by validator"),
    }
}

/// Generate builder code for a message
fn generate_builder(ir: &IrSchema, message: &IrMessage) -> String {
    let builder_name = format_ident!("{}", message.builder_name);
    let _reader_name = &message.reader_name;
    
    let fields = &message.fields;
    let field_methods: Vec<_> = fields
        .iter()
        .map(|field| generate_builder_field_method(ir, field))
        .collect();
    
    let new_method = generate_builder_new_method(message);
    let finish_method = generate_builder_finish_method(message);
    
    let code = quote! {
        #new_method
        
        impl #builder_name {
            #(#field_methods)*
            
            #finish_method
        }
        
        impl Default for #builder_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
    
    code.to_string()
}

/// Generate the new() method for builders
fn generate_builder_new_method(message: &IrMessage) -> TokenStream {
    let builder_name = format_ident!("{}", message.builder_name);
    
    quote! {
        /// Builder for #builder_name messages
        #[derive(Debug)]
        pub struct #builder_name {
            builder: MessageBuilder,
        }
        
        impl #builder_name {
            /// Create a new builder
            pub fn new() -> Self {
                Self {
                    builder: MessageBuilder::new(),
                }
            }
        }
    }
}

/// Generate the finish() method for builders
fn generate_builder_finish_method(message: &IrMessage) -> TokenStream {
    let builder_name = format_ident!("{}", message.builder_name);
    let reader_name = format_ident!("{}", message.reader_name);
    
    quote! {
        /// Finish building and return the serialized message
        pub fn finish(self) -> Vec<u8> {
            self.builder.finish()
        }
        
        /// Finish building and return a reader
        pub fn finish_reader(self) -> zeroproto::Result<#reader_name<'static>> {
            let bytes = self.finish();
            #reader_name::from_bytes(&bytes)
        }
    }
}

/// Generate a builder field method
fn generate_builder_field_method(_ir: &IrSchema, field: &IrField) -> TokenStream {
    let method_name = format_ident!("set_{}", field.rust_name);
    let field_index = field.index;
    
    match &field.field_type {
        IrFieldType::Scalar { rust_type, .. } => {
            let param_type = syn::parse_str::<syn::Type>(rust_type).unwrap();
            quote! {
                /// Set the #method_name field
                pub fn #method_name(&mut self, value: #param_type) -> &mut Self {
                    self.builder.set_scalar(#field_index, value).unwrap();
                    self
                }
            }
        }
        IrFieldType::UserDefined { rust_type, is_message, .. } => {
            if *is_message {
                quote! {
                    /// Set the #method_name field
                    pub fn #method_name(&mut self, value: &#rust_type<'_>) -> &mut Self {
                        let bytes = value.reader.finish();
                        self.builder.set_message(#field_index, &bytes).unwrap();
                        self
                    }
                }
            } else {
                let param_type = syn::parse_str::<syn::Type>(rust_type).unwrap();
                quote! {
                    /// Set the #method_name field
                    pub fn #method_name(&mut self, value: #param_type) -> &mut Self {
                        self.builder.set_scalar(#field_index, value).unwrap();
                        self
                    }
                }
            }
        }
        IrFieldType::Vector { element_type, .. } => {
            generate_vector_builder_method(field, element_type)
        }
    }
}

/// Generate vector builder method
fn generate_vector_builder_method(field: &IrField, element_type: &IrFieldType) -> TokenStream {
    let method_name = format_ident!("set_{}", field.rust_name);
    let field_index = field.index;
    
    match element_type {
        IrFieldType::Scalar { rust_type, .. } => {
            let param_type = syn::parse_str::<syn::Type>(&format!("[{}]", rust_type)).unwrap();
            quote! {
                /// Set the #method_name field
                pub fn #method_name(&mut self, values: &#param_type) -> &mut Self {
                    self.builder.set_vector(#field_index, values).unwrap();
                    self
                }
            }
        }
        IrFieldType::UserDefined { rust_type, is_message, .. } => {
            if *is_message {
                quote! {
                    /// Set the #method_name field
                    pub fn #method_name(&mut self, values: &[#rust_type<'_>]) -> &mut Self {
                        let bytes: Vec<_> = values.iter().map(|msg| {
                            let mut temp_builder = MessageBuilder::new();
                            // Copy all fields from the message
                            // This is a simplified version - in practice you'd want to copy all fields
                            msg.reader.finish()
                        }).collect();
                        self.builder.set_vector(#field_index, &bytes).unwrap();
                        self
                    }
                }
            } else {
                let param_type = syn::parse_str::<syn::Type>(&format!("[{}]", rust_type)).unwrap();
                quote! {
                    /// Set the #method_name field
                    pub fn #method_name(&mut self, values: &#param_type) -> &mut Self {
                        self.builder.set_vector(#field_index, values).unwrap();
                        self
                    }
                }
            }
        }
        IrFieldType::Vector { .. } => panic!("Nested vectors should have been caught by validator"),
    }
}
