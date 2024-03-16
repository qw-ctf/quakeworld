use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::{parse_macro_input, AngleBracketedGenericArguments, Attribute, DeriveInput, Meta, Type};
use syn::{Data, DataStruct, Fields};

#[proc_macro_derive(ParseMessage)]
pub fn parse_message_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_parsemessage_macro(&ast)
}

fn impl_parsemessage_macro(ast: &syn::DeriveInput) -> TokenStream {
    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let field_name = fields.iter().map(|field| &field.ident);
    let field_name_annotate = fields.iter().map(|field| {
        let ft = &field.ident;
        let q = quote! { #ft };
        format!("{}", q.to_string())
    });
    let field_name_value = fields.iter().map(|field| {
        let ft = &field.ident;
        let q = quote! { #ft };
        format_ident!("{}_value", q.to_string())
    });
    let field_name_value1 = fields.iter().map(|field| {
        let ft = &field.ident;
        let q = quote! { #ft };
        format_ident!("{}_value", q.to_string())
    });

    let field_function = fields.iter().map(|field| {
        let ft = &field.ty;
        let q = quote! { #ft };
        format_ident!("read_{}", q.to_string().to_lowercase())
    });

    let struct_name = &ast.ident;

    let gen = quote! {
        impl #struct_name {
            fn read(message: &mut Message) -> Result<ServerMessage, MessageError>
            {
                trace_start!(message, false);
                #(
                trace_annotate!(message, #field_name_annotate);
                let #field_name_value = message.#field_function(false)?;
                 )*
                let v = ServerMessage::#struct_name(
                        #struct_name{
                            #(
                                #field_name: #field_name_value1,
                                )*
                        });

                trace_stop!(message, v);
                Ok(v)
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(DataTypeRead, attributes(datatyperead))]
pub fn data_type_read_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Extract the name of the struct
    let struct_name = &ast.ident;

    // Extract field names and types
    let fields = if let syn::Data::Struct(data_struct) = &ast.data {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .map(|f| {
                    let ident = &f.ident;
                    let ty = &f.ty;

                    let qi = quote! {#ident};
                    let qt = quote! {#ty};
                    let vi = format!("{}", qi);
                    let mut v = format!("{}_{}", qi.to_string(), qt.to_string());
                    v = v.replace("<", "_");
                    v = v.replace(">", "_");
                    v = v.replace(" ", "_");

                    let id = format_ident!("{}", v);

                    (
                        quote! {
                        trace_annotate!(datareader, #vi);
                        let #id = <#ty as DataTypeRead>::read(datareader)?;
                        },
                        quote! {
                        #ident : #id,
                        },
                    )
                })
                .collect::<Vec<_>>()
        } else {
            panic!("DataTypeRead can only be derived for structs with named fields");
        }
    } else {
        panic!("DataTypeRead can only be derived for structs");
    };

    // Extract generic parameters
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let field_creation: Vec<_> = fields.iter().map(|(a, _)| a).collect();
    let field_assignment: Vec<_> = fields.iter().map(|(_, b)| b).collect();

    let (_, tag_value) = check_tag_value(&ast.attrs, "datatyperead");
    let datatype = match tag_value.get("prefix") {
        Some(p) => {
            let i = format_ident!(
                "{}{}",
                p.to_uppercase(),
                struct_name.to_string().to_uppercase()
            );
            quote! {DataType::#i(self)}
        }
        None => {
            let i = format_ident!("{}", struct_name.to_string().to_uppercase());
            quote! {DataType::#i(self)}
        }
    };
    println!("{}", datatype);

    // Generate the implementation
    let gen = quote! {
        impl #impl_generics DataTypeRead for #struct_name #ty_generics #where_clause {
            fn read(datareader: &mut DataTypeReader) -> Result<Self, DataTypeReaderError> {
                trace_start!(datareader, stringify!( #struct_name));
                #(#field_creation)*

                let s = Self {
                    #(#field_assignment)*
                };
                trace_stop!(datareader, s, #struct_name);
                Ok(s)
            }
            fn to_datatype(&self) -> DataType {
                paste! {
                DataType::None
                //DataType::[< #struct_name:upper >](self)
                }
            }
        }
    };

    // Return the generated implementation as a TokenStream
    gen.into()
}

#[proc_macro_derive(DataTypeBoundCheckDerive, attributes(check_bounds))]
pub fn data_type_bound_check_derive(input: TokenStream) -> TokenStream {
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

fn check_tag_value(attrs: &[Attribute], tag: &str) -> (bool, HashMap<String, String>) {
    let mut s: HashMap<String, String> = HashMap::new();
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
            if let syn::Meta::List(list) = meta {
                if list.path.is_ident(tag) {
                    // The specified attribute is present, extract its value
                    for nested_meta in list.nested {
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = nested_meta
                        {
                            if let Some(name) = name_value.path.get_ident() {
                                if let syn::Lit::Str(value) = name_value.lit {
                                    s.insert(name.to_string(), value.value());
                                }
                            }
                        }
                        return (true, s);
                    }
                }
            }
        }
    }
    (false, s)
}

fn check_tag(attrs: &[Attribute], tag: &str) -> bool {
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
            if let Meta::Path(name_value) = meta {
                if name_value.is_ident(tag) {
                    return true;
                }
            }
        }
    }
    false
}
