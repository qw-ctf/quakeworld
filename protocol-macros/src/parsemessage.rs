use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, DataStruct, Fields};

pub fn parsemessage_derive(input: TokenStream) -> TokenStream {
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
        format!("{}", q)
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
                trace::trace_start!(message, false);
                #(
                trace::trace_annotate!(message, #field_name_annotate);
                let #field_name_value = message.#field_function(false)?;
                 )*
                let v = ServerMessage::#struct_name(
                        #struct_name{
                            #(
                                #field_name: #field_name_value1,
                                )*
                        });

                trace::trace_stop!(message, v);
                Ok(v)
            }
        }
    };
    gen.into()
}
