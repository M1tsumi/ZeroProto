//! Procedural macros for ZeroProto

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for ZeroProto message types
#[proc_macro_derive(ZeroprotoMessage)]
pub fn derive_zeroproto_message(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// Create a new reader for this message
            pub fn reader<'a>(data: &'a [u8]) -> zeroproto::Result<zeroproto::MessageReader<'a>> {
                zeroproto::MessageReader::new(data)
            }

            /// Create a new builder for this message
            pub fn builder() -> zeroproto::MessageBuilder {
                zeroproto::MessageBuilder::new()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for ZeroProto field accessors
#[proc_macro_derive(ZeroprotoFields)]
pub fn derive_zeroproto_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Extract field information from struct attributes
    let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = input.data {
        fields
    } else {
        return TokenStream::from(
            quote! { compile_error!("ZeroprotoFields can only be derived on structs"); },
        );
    };

    let mut field_methods = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let field_name = &field.ident;
        let field_index = i as u16;

        // Generate accessor methods based on field type
        let field_type = &field.ty;

        let method = quote! {
            fn #field_name(&self) -> zeroproto::Result<#field_type> {
                self.get_scalar(#field_index)
            }
        };

        field_methods.push(method);
    }

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#field_methods)*
        }
    };

    TokenStream::from(expanded)
}
