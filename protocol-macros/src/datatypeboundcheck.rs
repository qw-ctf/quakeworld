use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::helpers::*;

pub fn datatype_bound_check_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);
    // Extract the name of the struct
    let struct_name = &ast.ident;

    // Extract field names, types, and tags
    let fields = if let syn::Data::Struct(data_struct) = &ast.data {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .map(|f| {
                    let ident = &f.ident;
                    let tag = check_tag(&f.attrs, "check_bounds");
                    if tag {
                        quote! {
                            self.#ident.check_bounds(datareader)?;
                        }
                    } else {
                        quote! {}
                    }
                })
                .collect::<Vec<_>>()
        } else {
            panic!("DataTypeRead can only be derived for structs with named fields");
        }
    } else {
        panic!("DataTypeRead can only be derived for structs");
    };

    // Generate the implementation
    let gen = quote! {
        impl DataTypeBoundCheck for #struct_name {
            fn check_bounds(&self, datareader: &mut DataTypeReader) -> Result<(), DataTypeReaderError> {
                #(#fields)*
                Ok(())
            }
        }
    };

    // Return the generated implementation as a TokenStream
    gen.into()
}
