

use syn::{
    braced, parenthesized, parse::{Parse, ParseStream}, punctuated::Punctuated, spanned::Spanned, token, Attribute, Expr, PathSegment, Token
};

use proc_macro::{Delimiter, Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_macro_input, DeriveInput, Type};

use crate::helpers::*;
use syn::parse::Parser;

/*
* we need to support:
* size_from = int // set the size to the int
* size_from = String // pull the size from the environment
* offset_from = int // set the offset to the int
* offset_from = String // pull the offset from the environment
*/


#[derive(Default, PartialEq, Eq)]
enum OffsetParsed {
    #[default]
    None,
    Int(syn::LitInt),
    Str(syn::LitStr),
}

impl From<&SizeOffset> for OffsetParsed {
    fn from(value: &SizeOffset) -> Self {
        match value {
            SizeOffset::None| SizeOffset::SizeInt(_) |SizeOffset::SizeStr(_) => OffsetParsed::None,
            SizeOffset::OffsetInt(lit_int) => OffsetParsed::Int(lit_int.clone()),
            SizeOffset::OffsetStr(lit_str) => OffsetParsed::Str(lit_str.clone()),
            SizeOffset::SizeOffsetStrStr(lit_str, _)|  SizeOffset::SizeOffsetStrInt(lit_str, _) => OffsetParsed::Str(lit_str.clone()),
            SizeOffset::SizeOffsetIntStr(lit_int, _) | SizeOffset::SizeOffsetIntInt(lit_int, _) => OffsetParsed::Int(lit_int.clone()),
            SizeOffset::SizeOffsetStr(lit_str) => {
                    let name_offset = format!("{}_offset", lit_str.value());
                    let name_offset = syn::LitStr::new(&name_offset, lit_str.span());
                OffsetParsed::Str(name_offset)
            },
        }
    }
}

#[derive(Default, PartialEq, Eq)]
enum SizeParsed {
    #[default]
    None,
    Int(syn::LitInt),
    Str(syn::LitStr),
}

impl From<&SizeOffset> for SizeParsed {
    fn from(value: &SizeOffset) -> Self {
        match value {
            SizeOffset::None| SizeOffset::OffsetInt(_) |SizeOffset::OffsetStr(_) => SizeParsed::None,
            SizeOffset::SizeInt(lit_int) => SizeParsed::Int(lit_int.clone()),
            SizeOffset::SizeStr(lit_str) => SizeParsed::Str(lit_str.clone()),
            SizeOffset::SizeOffsetStrStr(_, lit_str) | SizeOffset::SizeOffsetIntStr(_, lit_str) => SizeParsed::Str(lit_str.clone()),
            SizeOffset::SizeOffsetIntInt(_, lit_int) | SizeOffset::SizeOffsetStrInt(_, lit_int) => SizeParsed::Int(lit_int.clone()),
            SizeOffset::SizeOffsetStr(lit_str) => {
                let name_size= format!("{}_size", lit_str.value());
                let name_size= syn::LitStr::new(&name_size, lit_str.span());
                SizeParsed::Str(name_size)},
        }
    }
}

#[derive(Default)]
struct FieldAttributesParsed {
    pub treat_as_string: bool,
    pub size: SizeParsed,
    pub offset: OffsetParsed,
    pub environment: Environment,
}


#[derive(Debug, Clone, Default)]
enum StructDataType {
    #[default]
    None,
    String(syn::LitStr),
    Ident(syn::Ident),

}

#[derive(Debug, Default, Clone)]
struct StructAttr {
    pub prefix: Option<syn::LitStr>, // prefix for the struct
    pub datatype: StructDataType, // datatype overwrite for the struct
}

impl Parse for StructAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut struct_attributes = StructAttr::default();

        loop {
            if input.is_empty() {
                break;
            }
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
            if input.is_empty() {
                break;
            }
            let type_name: syn::Ident = input.parse()?;
            match type_name.to_string().as_str() {
                "prefix" => {
                    if input.peek(Token![=]) {
                        let _: Option<Token![=]> = input.parse()?;
                        if input.peek(syn::LitStr) {
                            let s: syn::LitStr = input.parse()?;
                            struct_attributes.prefix = Some(s);
                            continue;
                        }
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value", type_name)))
                    } else {
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value", type_name)))
                    }
                },
                "datatype" => {
                    if input.peek(Token![=]) {
                        let _: Option<Token![=]> = input.parse()?;
                        if input.peek(syn::LitStr) {
                            let s: syn::LitStr = input.parse()?;
                            struct_attributes.datatype = StructDataType::String(s.clone());
                            continue;
                        }

                        if input.peek(syn::Ident) {
                            let s: syn::Ident = input.parse()?;
                            struct_attributes.datatype = StructDataType::Ident(s.clone());
                            continue;
                        }

                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value", type_name)))
                    } else {
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value", type_name)))
                    }
                },
                _ => {
                    return Err(syn::Error::new(type_name.span(), format!("`{}` attribute not supported", type_name)))
                },
            }
        }
        Ok(struct_attributes)
    }
}

#[derive(Debug, Clone, Default)]
enum SizeOffset {
    #[default]
    None,
    SizeInt(syn::LitInt),
    SizeStr(syn::LitStr),
    OffsetInt(syn::LitInt),
    OffsetStr(syn::LitStr),
    SizeOffsetStrStr(syn::LitStr, syn::LitStr),
    SizeOffsetStrInt(syn::LitStr, syn::LitInt),
    SizeOffsetIntStr(syn::LitInt, syn::LitStr),
    SizeOffsetIntInt(syn::LitInt, syn::LitInt),
    SizeOffsetStr(syn::LitStr),
}

#[derive(Debug, Clone, Default)]
enum Environment {
    #[default]
    None,
    Auto,
    String(syn::LitStr)
}

#[derive(Debug, Clone)]
enum FieldAttribute {
    SizeOffset(SizeOffset),
    EnvironmentSet(Environment),
    String, // Parse a Vec<u8> as a string
}

#[derive(Debug)]
struct FieldAttr {
    pub attributes: Vec<FieldAttribute>,
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut field_attributes: Vec<FieldAttribute> = Vec::new();
        loop {
            if input.is_empty() {
                break;
            }
            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            }
            if input.is_empty() {
                break;
            }
            let type_name: syn::Ident = input.parse()?;
            match type_name.to_string().as_str() {
                "environment" => {
                    if input.peek(Token![=]) {
                        let _: Option<Token![=]> = input.parse()?;
                        if input.peek(syn::LitStr) {
                            let s: syn::LitStr = input.parse()?;
                            field_attributes.push(FieldAttribute::EnvironmentSet(Environment::String(s)));
                            continue;
                        }
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value", type_name)))
                    } else {
                        field_attributes.push(FieldAttribute::EnvironmentSet(Environment::Auto));
                        continue;
                    }
                },
                "string" => {
                    field_attributes.push(FieldAttribute::String);
                },
                "size_from" => {
                    if input.peek(Token![=]) {
                        let _: Option<Token![=]> = input.parse()?;
                        if input.peek(syn::LitInt) {
                            let i: syn::LitInt = input.parse()?;
                            field_attributes.push(
                                FieldAttribute::SizeOffset(
                                    SizeOffset::SizeInt(i)));
                            continue;
                        }
                        if input.peek(syn::LitStr) {
                            let s: syn::LitStr = input.parse()?;
                            field_attributes.push(
                                FieldAttribute::SizeOffset(
                                    SizeOffset::SizeStr(s)));
                            continue;
                        }
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value", type_name)))
                    } else {
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute needs a = value", type_name)))
                    }
                },
                "offset_from" => {
                    if input.peek(Token![=]) {
                        let _: Option<Token![=]> = input.parse()?;
                        if input.peek(syn::LitInt) {
                            let i: syn::LitInt = input.parse()?;
                            field_attributes.push(
                                FieldAttribute::SizeOffset(
                                    SizeOffset::OffsetInt(i)));
                            continue;
                        }
                        if input.peek(syn::LitStr) {
                            let s: syn::LitStr = input.parse()?;
                            field_attributes.push(
                                FieldAttribute::SizeOffset(
                                    SizeOffset::OffsetStr(s)));
                            continue;
                        }
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value", type_name)))
                    } else {
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute needs a = value", type_name)))
                    }
                },
                "size_offset_from" => {
                    if input.peek(Token![=]) {
                        let _: Option<Token![=]> = input.parse()?;
                        if input.peek(token::Paren) {
                            // we are in parenthesis
                            let content;
                            let _ = parenthesized!(content in input);
                            if content.peek(syn::LitInt) {
                                let first_field_int: syn::LitInt = content.parse()?;
                                let _: Token![,] = content.parse()?;
                                if content.peek(syn::LitInt) {
                                    let second_field_int: syn::LitInt = content.parse()?;
                                    field_attributes.push(
                                        FieldAttribute::SizeOffset(
                                            SizeOffset::SizeOffsetIntInt(
                                                first_field_int,
                                                second_field_int)));
                                    continue;
                                }
                                if content.peek(syn::LitStr) {
                                    let second_field_str: syn::LitStr = content.parse()?;
                                    field_attributes.push(
                                        FieldAttribute::SizeOffset(
                                            SizeOffset::SizeOffsetIntStr(
                                                first_field_int,
                                                second_field_str)));
                                    continue;
                                }
                                return Err(syn::Error::new(type_name.span(), format!("`{}` attribute first value supplied is wrong", type_name)))
                            } else if content.peek(syn::LitStr) {
                                let first_field_str: syn::LitStr = content.parse()?;
                                if content.peek(syn::LitInt) {
                                    let second_field_int: syn::LitInt = content.parse()?;
                                    field_attributes.push(
                                        FieldAttribute::SizeOffset(
                                            SizeOffset::SizeOffsetStrInt(
                                                first_field_str,
                                                second_field_int)));
                                    continue;
                                }
                                if content.peek(syn::LitStr) {
                                    let second_field_str: syn::LitStr = content.parse()?;
                                    field_attributes.push(
                                        FieldAttribute::SizeOffset(
                                            SizeOffset::SizeOffsetStrStr(
                                                first_field_str,
                                                second_field_str)));
                                    continue;
                                }
                                return Err(syn::Error::new(type_name.span(), format!("`{}` attribute second value supplied is wrong", type_name)))
                            }
                            return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value supplied", type_name)))
                        } else if input.peek(syn::LitInt) {
                            return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value supplied", type_name)))
                        } else if input.peek(syn::LitStr) {
                            let s = input.parse()?;
                            field_attributes.push(
                                FieldAttribute::SizeOffset(
                                    SizeOffset::SizeOffsetStr(s)));
                            continue;
                        }
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute does not support the value supplied", type_name)))
                    } else {
                        return Err(syn::Error::new(type_name.span(), format!("`{}` attribute needs a = value", type_name)))
                    }
                },
                _ => {
                    return Err(syn::Error::new(type_name.span(), format!("`{}` attribute not supported", type_name)))
                },
            }
        }
        Ok(FieldAttr { attributes: field_attributes})
    }
}


#[derive(Debug)]
struct FieldInformation {
    pub identifier: syn::Ident,
    pub ty: syn::Type,
    pub attributes: Vec<FieldAttribute>,
}

#[derive(Debug)]
struct StructInformation {
    pub identifier: syn::Ident,
    pub generics: syn::Generics,
    pub is_generic: bool,
    pub fields: Vec<FieldInformation>,
    pub attributes: StructAttr,
}

pub fn datatyperead_derive_2(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // check if we are implementing on a struct
    let data_struct = match input.data {
        syn::Data::Struct(data_struct) => data_struct,  
        syn::Data::Enum(data_enum) => {
            return quote_spanned! {
                data_enum.enum_token.span =>
                compile_error!("DataTypeReader can only be used on structs not enums");
            }.into()
        },
        syn::Data::Union(data_union) => {
            return quote_spanned! {
                data_union.union_token.span =>
                compile_error!("DataTypeReader can only be used on structs not unions");
            }.into()
        },
    };

    let mut struct_attrib=  StructAttr::default();
    for attr in &input.attrs {
        if !attr.path.is_ident("datatyperead"){continue;}
        // println!("{:?}", attr);

        let attr_span = attr.span();
        let a = attr.parse_args_with(Punctuated::<StructAttr, Token![,]>::parse_terminated);
        if let Ok(sa) = a { struct_attrib = sa[0].clone() };
    }

    if input.ident.clone() == "SizedVectorNameTest" {
        println!("we are here right: {:?} {}", input.generics.params.first(), input.generics.params.first().is_some());
    }

    let is_generic = input.generics.params.first().is_some();

    let mut struct_information = StructInformation{
        identifier: input.ident.clone(),
        generics: input.generics,
        is_generic,
        fields: vec![],
        attributes: struct_attrib,
    };
    // println!("{}", input.ident);
    // println!("{}", input.generics.params.to_token_stream());
    // println!("{}", struct_information);
    for field in data_struct.fields.iter() {
        let field_span= field.span();
        let field_ident = match &field.ident {
            Some(f) => f.clone(),
            None => {
                return quote_spanned! {
                    field_span =>
                    compile_error!("DataTypeReader can only be used on structs with named fields");
                }.into()
            },
        };

        let field_ty= field.ty.clone();

        let mut field_attributes: Vec<FieldAttribute>= vec![];

        for attr in &field.attrs {
            if !attr.path.is_ident("datatyperead"){continue;}
            let attr_span = attr.span();
            let a = attr.parse_args_with(Punctuated::<FieldAttr, Token![,]>::parse_terminated);
            match a {
                Ok(a) => {
                    for attribute in a[0].attributes.clone() {
                        field_attributes.push(attribute);
                    }
                },
                Err(e) => {
                    let b = format!("{}", e);
                    return quote_spanned! {
                        attr_span =>
                        compile_error!(#b);
                    }.into();
                },
            };
        }
        struct_information.fields.push(FieldInformation {
            identifier: field_ident,
            ty: field_ty,
            attributes: field_attributes
        });
    }
    // println!("we here?: {:?}", struct_information);

    let prefix = match &struct_information.attributes.prefix {
        Some(e) => e.value().clone().to_uppercase(),
        None => "".to_string(),
    };
    let struct_name = struct_information.identifier.clone().to_string().to_uppercase();
    // let (_, generic_types, _) = struct_information.generics.split_for_impl();
    let datatype = match struct_information.generics.params.first() {
        Some(_) => format_ident!("{}{}GENERIC", prefix, struct_name)
,
        None => format_ident!("{}{}", prefix, struct_name)
    };

    let mut field_creations: Vec<_> = vec![];
    let mut field_assignments: Vec<_> = vec![];
    let mut field_errors: Vec<_> = vec![];

    struct_information.fields.iter().for_each(|f| {
        let mut field_creation: Vec<_> = vec![];
        let mut field_assignment: Vec<_> = vec![];
        let mut field_error: Vec<_> = vec![];

        // println!("parsing field: {}", f.identifier);

        // yes i am aware
        let mut fap = FieldAttributesParsed::default();

        // let mut field_attributes: Vec<TokenStream> = vec![];
        for attribute in &f.attributes {
            match attribute {
                FieldAttribute::SizeOffset(size_offset) => {
                    let s:SizeParsed  = size_offset.into();
                    if fap.size != SizeParsed::None && s != SizeParsed::None {
                        field_error.push(quote_spanned! {
                            f.identifier.span() =>
                            compile_error!("Size set twice");
                        });
                    } else if fap.size == SizeParsed::None {
                        fap.size = s;
                    }

                    let o:OffsetParsed = size_offset.into();
                    if fap.offset != OffsetParsed::None && o != OffsetParsed::None {
                        field_error.push(quote_spanned! {
                            f.identifier.span() =>
                            compile_error!("Offset set twice");
                        });
                    } else if fap.offset == OffsetParsed::None {
                        fap.offset = o;
                    }
                },
                FieldAttribute::EnvironmentSet(environment) => fap.environment = environment.clone(),
                FieldAttribute::String => fap.treat_as_string = true,
            }
        }
        let field_name = f.identifier.clone();
        let field_identifier = format_ident!("{}_identifier", f.identifier.clone());
        let field_type = f.ty.clone();
        // let newline = format_ident!("\n");
        // let newline = Char::from_u8(10).unwrao();

        let read_exect_type = match fap.treat_as_string  {
            true => quote!{ read_exact_generic_string },
            false => quote!{ read_exact_generic_v2 },
        };

        let field_environment = match fap.environment {
            Environment::None => quote!{},
            Environment::Auto => quote!{
                #field_identifier . environment( datareader , stringify!(#field_name));
            },
            Environment::String(lit_str) => quote!{
                #field_identifier . environment( datareader , #lit_str);
            },
        };

        let mut field_offset_after = quote!{};
        let field_offset = match fap.offset {
            OffsetParsed::None => quote!{},
            OffsetParsed::Int(lit_int) => {
                field_offset_after = quote!{
                    datareader.set_position(old_offset);
                };
                quote!{
                    let old_offset = datareader.position();
                    datareader.set_position(#lit_int);
                }},
            OffsetParsed::Str(lit_str) => {
                field_offset_after = quote!{
                    datareader.set_position(old_offset);
                };
                quote!{
                    let old_offset = datareader.position();
                    let current_field_offset: u64 = datareader.get_env_error(#lit_str)?.into();
                    datareader.set_position(current_field_offset);
                }},
        };

        // Generating field reading
        let fc = match fap.size {
            SizeParsed::None => quote!{
                #field_offset
                let #field_identifier = <#field_type as DataTypeRead>::read(datareader)?;
                #field_offset_after
                #field_environment
            },
            SizeParsed::Int(lit_int) => quote!{
                let size_from_environment: usize = #lit_int;
                let mut #field_identifier: #field_type = Vec::with_capacity(size_from_environment);
                #field_offset
                datareader. #read_exect_type (&mut #field_identifier)?;
                #field_offset_after
                #field_environment
            },
            SizeParsed::Str(lit_str) => quote!{
                let size_from_environment: usize = datareader.get_env_error(#lit_str)? .into();
                let mut #field_identifier: #field_type = Vec::with_capacity(size_from_environment);
                #field_offset
                datareader.read_exact_generic_v2(&mut #field_identifier)?;
                #field_offset_after
                #field_environment
            },
        };

        // let fc = match f.identifier { quote!{
        //     // let #field_identifier: #field_type = datareader.read()?;
        //     let #field_identifier = <#field_type as DataTypeRead>::read(datareader)?;
        // }.to_token_stream();}

        let fa = quote!{
            #field_name: #field_identifier,
        }.to_token_stream();
        // println!("\t\tfa: {}", fa);
        // println!("\t\tfc: {}", fc);
        field_creation.push(fc);
        field_assignment.push(fa);
        // println!("\tfield_creation: {:?}", field_creation);
        // println!("\tfield_assignment: {:?}", field_assignment);

        let _ = field_error.into_iter().map(|f| field_errors.push(f));

        for f in field_creation.into_iter() {
            field_creations.push(f);
        }

        for f in field_assignment.into_iter() {
            field_assignments.push(f);
        }
        // let _ = field_creation.into_iter().map(|f| field_creations.push(f));
        // field_creations.push(field_creation.into_iter().collect());
        // field_assignments.push(field_assignment.into_iter().collect());
        // let _ = field_assignment.into_iter().map(|f| field_assignments.push(f));

        // println!("\tfield_creations: {:?}", field_creations);
        // println!("\tfield_assignments: {:?}", field_assignments);
    });

    // println!("field_creations: {:?}", field_creations);
    // println!("field_assignments: {:?}", field_assignments);
    // for f in field_errors {}
    // let field_errors = field_errors.iter().map(|a| a).collect();
    let (struct_impl_generics, struct_type_generics, struct_where_clause) = struct_information.generics.split_for_impl();

    let si_identifier = {
        let ident = struct_information.identifier.clone();
        quote!{ #ident }
    };

    let struct_name = match struct_information.attributes.prefix {
        Some(p) => {
            // let prefix = format_ident!("{}", p.value());
            quote!{ #prefix::#p }},
        None => quote!{#si_identifier},
    };

    let has_data = match struct_information.is_generic {
        false => quote!{ (self.clone())},
        true => quote!{},
    };

    let datatype_overwrite = match struct_information.attributes.datatype {
        StructDataType::None => quote!{ DataType :: #datatype #has_data },
        StructDataType::String(ident) => quote!{ DataType :: #ident },
        StructDataType::Ident(ident) => quote!{ DataType :: #ident },
        // None => quote!{ DataType :: #datatype #has_data },
    };
    let gen = quote! {
        impl #struct_impl_generics DataTypeRead for #si_identifier #struct_type_generics #struct_where_clause {
            fn read(datareader: &mut DataTypeReader) -> Result<Self, DataTypeReaderError> {
                trace_start!(datareader, stringify!( #si_identifier));

                #(#field_errors)*
                #(#field_creations)*

                let s = Self {
                    #(#field_assignments)*
                };
                // trace_stop!(datareader, s, #struct_name);
                Ok(s)
            }

            fn to_datatype(&self) -> DataType {
                #datatype_overwrite
            }
        }
    };

    // println!("{}", struct_name);
    if struct_name.to_string() == "SizedVectorNameTest" {
        println!("{}", gen);
    }
    gen.into()
}


#[derive(Debug)]
struct DataTypeTypeStructInfo {
    pub name: syn::Ident,
    pub generic: bool,
}

#[derive(Default, Debug)]
enum SizeFrom {
    #[default]
    None,
}

#[derive(Default, Debug)]
enum OffsetFrom {
    #[default]
    None,
}

#[derive(Default, Debug)]
enum SizeOffsetFrom {
    #[default]
    None,
}

#[derive(Default, Debug)]
enum SizeOffsetFromTypes {
    #[default]
    None,
    Size(SizeFrom),
    Offset(OffsetFrom),
    SizeOffset(SizeOffsetFrom),
}

#[derive(Default, Debug)]
enum SetEnvironment {
    #[default]
    None,
    Auto,
    DirectoryEntry,
    String(String)
}

#[derive(Default, Debug)]
struct SetSizeFromDirectoryEntry {
    pub active: bool,
    pub name: String,
}

#[derive(Default, Debug)]
enum SizeFromEnvironmentType {
    #[default]
    None,
    String(String),
    Int(usize),
    DirectoryEntry(Option<String>)
}

#[derive(Default, Debug)]
struct SizeFromEnvironment {
    pub active: bool,
    pub parse_as_string: bool,
    pub from: SizeFromEnvironmentType,
}

struct FieldEntry {
    pub t: Type,
    pub name: String,
    pub generic_field_type: String,
    pub identifier: syn::Ident,
    pub identifier_formated: syn::Ident,
}

/// derive macro for DataTypeRead
///
/// available attributes:
///   string : the element is to be read as a string if no size attribute is set it will read till
///   a 0 byte is encountered. if size is a
pub fn datatyperead_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let ast = parse_macro_input!(input as DeriveInput);

    // Extract the name of the struct
    let struct_name = &ast.ident;
    let mut dtsi = DataTypeTypeStructInfo{ name: ast.ident.clone(), generic: false };


    if let Some(param) = ast.generics.params.first() {
        match param {
            syn::GenericParam::Type(type_param) => {
                dtsi.generic = true;
            },
            syn::GenericParam::Lifetime(lifetime_def) => {},
            syn::GenericParam::Const(const_param) => {},
        }
    };
    // if let syn::Type::Path(path) = ast     // Extract field names and types
    let fields = if let syn::Data::Struct(data_struct) = &ast.data {
        if let syn::Fields::Named(fields) = &data_struct.fields {
            fields
                .named
                .iter()
                .map(|f| {
                    let ft = &f.ty;
                    let identifier = f.ident.clone().unwrap();
                    let quoted_type = quote! {#ft};
                    let identifier_formated= format_ident!("{}", format!("{}_{}", identifier , quoted_type).replace(&[ '<', '>', ' ', ':' ][..], "_"));
                    let mut current_field = FieldEntry{
                        t: f.ty.clone(),
                        name: f.ident.clone().unwrap().to_string(),
                        generic_field_type: "".to_string(),
                        identifier,
                        identifier_formated,
                    };

                    // extract generic type
                    if let Type::Path(path) = ft {
                        for segment in &path.path.segments {
                            if let arguments = &segment.arguments {
                                if let syn::PathArguments::AngleBracketed(args) = arguments {
                                    for arg in &args.args {
                                        if let syn::GenericArgument::Type(t) = arg {
                                            if let syn::Type::Path(p) = t {
                                                for seg in &p.path.segments {
                                                    if let Some(s) = seg.ident.span().source_text() {
                                                        current_field.generic_field_type.push_str(&s as &str);
                                                    }
                                                }
                                            }

                                        }
                                    }
                                }
                            }
                        }
                    }

                    let mut size_from_environment = SizeFromEnvironment::default();
                    // check if we have datatype read attributes
                    // if string and size is set reading will stop at the first \0
                    let tag_value = get_tag_values(&f.attrs, "datatyperead");

                    size_from_environment.active =
                        tag_value.contains_key("string") ||
                        tag_value.contains_key("size") ||
                        tag_value.contains_key("size_from_directory_entry");

                    let mut set_size_from_environment = SetSizeFromDirectoryEntry::default();
                    set_size_from_environment.active = tag_value.contains_key("environment");
                    let set_environment = match tag_value.get("environment") {
                        Some(v) => match v {
                            syn::Lit::Verbatim(literal) => todo!(),
                            syn::Lit::ByteStr(lit_byte_str) => todo!(),
                            syn::Lit::Byte(lit_byte) => todo!(),
                            syn::Lit::Char(lit_char) => todo!(),
                            syn::Lit::Int(lit_int) => todo!(),
                            syn::Lit::Float(lit_float) => todo!(),
                            syn::Lit::Str(lit_str) => SetEnvironment::String(lit_str.value()),
                            syn::Lit::Bool(lit_bool) => SetEnvironment::Auto,
                        },
                        None => SetEnvironment::None,
                    };

                    size_from_environment.parse_as_string = tag_value.contains_key("string");

                    if let Some(v) = tag_value.get("size_from_directory_entry") {
                        match  v {
                            syn::Lit::Str(lit_str) => panic!("Str"),
                            syn::Lit::ByteStr(lit_byte_str) => panic!("ByteStr"),
                            syn::Lit::Byte(lit_byte) => panic!("Byte"),
                            syn::Lit::Char(lit_char) => panic!("Char"),
                            syn::Lit::Int(lit_int) => panic!("Int"),
                            syn::Lit::Float(lit_float) => panic!("Float"),
                            syn::Lit::Bool(lit_bool) => {
                                size_from_environment.from = SizeFromEnvironmentType::DirectoryEntry(None);
                            },
                            syn::Lit::Verbatim(literal) => panic!("Verbatim"),
                            // syn::Lit::Str(value) => {
                            //     size_from_environment.from = SizeFromEnvironmentType::DirectoryEntry(value.value());
                            // },
                            // _ =>  {
                            //     panic!("we only support strings for now")
                            // }
                        }
                    }

                    let allowed_types = vec!["size_from", "offset_from", "size_offset_from"];
                    let mut types = vec![];
                    for t in allowed_types {
                        if tag_value.contains_key(t) {
                            types.push(t);
                        }
                    }
                    if types.len() > 1 {
                        panic!("only one *_from parameter allowed. found ({})", types.join(","));
                    }

                    let mut size_offset_from = SizeOffsetFromTypes::None;
                    if types.len() == 1 {
                        if let Some(v) = tag_value.get(types[0]) {
                            match types[0]{
                                "size_offset_from" => {
                                    size_offset_from = SizeOffsetFromTypes::SizeOffset(SizeOffsetFrom::None);
                                    match v {
                                        syn::Lit::Str(lit_str) => panic!("Str"),
                                        syn::Lit::ByteStr(lit_byte_str) => panic!("ByteStr"),
                                        syn::Lit::Byte(lit_byte) => panic!("Byte"),
                                        syn::Lit::Char(lit_char) => panic!("Char"),
                                        syn::Lit::Int(lit_int) => panic!("Int"),
                                        syn::Lit::Float(lit_float) => panic!("Float"),
                                        syn::Lit::Bool(lit_bool) => panic!("Bool"),
                                        syn::Lit::Verbatim(literal) => panic!("Verbatim"),
                                    }
                                },
                                "size_from" => {
                                    size_offset_from = SizeOffsetFromTypes::Size(SizeFrom::None);
                                    match v {
                                        syn::Lit::Str(lit_str) => panic!("Str"),
                                        syn::Lit::ByteStr(lit_byte_str) => panic!("ByteStr"),
                                        syn::Lit::Byte(lit_byte) => panic!("Byte"),
                                        syn::Lit::Char(lit_char) => panic!("Char"),
                                        syn::Lit::Int(lit_int) => panic!("Int"),
                                        syn::Lit::Float(lit_float) => panic!("Float"),
                                        syn::Lit::Bool(lit_bool) => panic!("Bool"),
                                        syn::Lit::Verbatim(literal) => panic!("Verbatim"),
                                    }
                                },
                                "offset" => {
                                    size_offset_from = SizeOffsetFromTypes::Offset(OffsetFrom::None);
                                    match v {
                                        syn::Lit::Str(lit_str) => panic!("Str"),
                                        syn::Lit::ByteStr(lit_byte_str) => panic!("ByteStr"),
                                        syn::Lit::Byte(lit_byte) => panic!("Byte"),
                                        syn::Lit::Char(lit_char) => panic!("Char"),
                                        syn::Lit::Int(lit_int) => panic!("Int"),
                                        syn::Lit::Float(lit_float) => panic!("Float"),
                                        syn::Lit::Bool(lit_bool) => panic!("Bool"),
                                        syn::Lit::Verbatim(literal) => panic!("Verbatim"),
                                    }
                                },

                                _ => panic!("{} should not be here", types[0]),
                            }

                        }
                    }

                    let mut size_offset_from = SizeOffsetFromTypes::default();
                    if let Some(v) = tag_value.get("size_offset_from") {
                        match  v {
                            syn::Lit::Str(lit_str) => panic!("Str"),
                            syn::Lit::ByteStr(lit_byte_str) => panic!("ByteStr"),
                            syn::Lit::Byte(lit_byte) => panic!("Byte"),
                            syn::Lit::Char(lit_char) => panic!("Char"),
                            syn::Lit::Int(lit_int) => panic!("Int"),
                            syn::Lit::Float(lit_float) => panic!("Float"),
                            syn::Lit::Bool(lit_bool) => {
                                // size_from_environment.from = SizeFromEnvironmentType::DirectoryEntry(None);
                                panic!("bool");
                            },
                            syn::Lit::Verbatim(literal) => panic!("Verbatim"),
                        }
                    }

                    if let Some(v) = tag_value.get("size") {
                        size_from_environment.active = true;
                        match v {
                            syn::Lit::Str(lit_str) => {
                                size_from_environment.from = SizeFromEnvironmentType::String(lit_str.value());
                            },
                            syn::Lit::ByteStr(lit_byte_str) => todo!(),
                            syn::Lit::Byte(lit_byte) => todo!(),
                            syn::Lit::Char(lit_char) => todo!(),
                            syn::Lit::Int(lit_int) => 
                            {
                                if let Ok(parsed_value) = lit_int.base10_parse::<usize>() {
                                    size_from_environment.from = SizeFromEnvironmentType::Int(parsed_value);
                                } else {
                                    panic!(
                                    "datatypereader attribute size's value couldnt be converted to usize"
                                );
                                };
                            },
                            syn::Lit::Float(lit_float) => todo!(),
                            syn::Lit::Bool(lit_bool) => todo!(),
                            syn::Lit::Verbatim(literal) => todo!(),
                        }
                    }

                    let field_type = &current_field.t;
                    let field_type_string = format!("{}", current_field.identifier);
                    let field_identifier = &current_field.identifier_formated;

                    if set_size_from_environment.active {
                        set_size_from_environment.name = field_type_string.clone();
                    }

                    let set_size_code = match set_size_from_environment.active {
                        true => {
                            let no = format!("{}_offset", field_type_string);
                            let ns = format!("{}_size", field_type_string);
                            let r = quote!{
                                let s = #field_identifier.offset as i64;
                                datareader.set_env(#no,s);
                                let s = #field_identifier.size as i64;
                                datareader.set_env(#ns, s); 
                            };
                            r

                        },
                        false => quote!{},
                    };

                    // println!("field_type: {}", quote!{#field_type});
                    let set_size_code = match set_environment {
                        SetEnvironment::None => quote!{},
                        SetEnvironment::Auto => {
                            let fts = quote!{#field_type}.to_string();
                            match fts.as_str() {
                                "DirectoryEntry" => {
                                    let no = format!("{}_offset", field_type_string);
                                    let ns = format!("{}_size", field_type_string);
                                    quote!{
                                        let s = #field_identifier.offset as i64;
                                        datareader.set_env(#no,s);
                                        let s = #field_identifier.size as i64;
                                        datareader.set_env(#ns, s); 
                                    }
                                },
                                _ => {
                                    quote!{
                                        datareader.set_env(#field_type_string, #field_identifier as i64);
                                    }
                                },
                            }

                        },
                        SetEnvironment::DirectoryEntry => todo!(),
                        SetEnvironment::String(name) => {
                            quote!{
                                datareader.set_env(#name, #field_identifier as i64);
                            }
                        },
                    };
                    // println!("+++ set_size_code:");
                    // println!("{}", set_size_code);
                    // println!("--- set_size_code:");

                    let read =
                    match size_from_environment.from {
                        SizeFromEnvironmentType::None => 
                        quote! {
                            trace_annotate!(datareader, #field_type_string);
                            let #field_identifier = <#field_type as DataTypeRead>::read(datareader)?;
                            #set_size_code
                        } ,
                        SizeFromEnvironmentType::String(size_env) => {
                            let read_type = format_ident!("read_exact_generic_v2");
                            // let size_env = format!(r#""{}""#, size_env);
                            quote! {
                                trace_annotate!(datareader, #field_type_string);
                                let size: usize = match datareader.get_env(#size_env) {
                                    Some(value) => {
                                        value.into()
                                    }
                                    None => {
                                        return Err(
                                            DataTypeReaderError::EnvironmentVariableNotSet(
                                                #size_env.to_string(),
                                                stringify!(#struct_name).to_string()));
                                    }
                                };
                                let mut #field_identifier: #field_type = Vec::with_capacity(size);
                                datareader. #read_type (&mut #field_identifier)?;
                            }
                        },
                        SizeFromEnvironmentType::Int(size) => {
                            let read_type = if size_from_environment.parse_as_string {
                                format_ident!("read_exact_generic_string")
                            } else {
                                format_ident!("read_exact_generic")
                            };
                            quote! {
                                trace_annotate!(datareader, #field_type_string);
                                let mut #field_identifier: #field_type = Vec::with_capacity(#size);
                                datareader. #read_type (&mut #field_identifier)?;
                            }
                        },
                        SizeFromEnvironmentType::DirectoryEntry(name)=> {

                            let field_type_string = format!("{}", current_field.identifier);
                            let env_size = format!("{}_size", field_type_string);
                            let env_offset = format!("{}_offset", field_type_string);
                            let read_type = format_ident!("read_exact_generic_v2");
                            let generic_field_type = format_ident!("{}", current_field.generic_field_type);
                            quote! {
                                trace_annotate!(datareader, #field_type_string);
                                let size: usize = match datareader.get_env(#env_size) {
                                    Some(value) => {
                                        value.into()
                                    }
                                    None => {
                                        return Err(
                                            DataTypeReaderError::EnvironmentVariableNotSet(
                                                stringify!(#env_size).to_string(),
                                                stringify!(#struct_name).to_string()));
                                    }
                                };

                                let offset: usize = match datareader.get_env(#env_offset) {
                                    Some(value) => {
                                        value.into()
                                    }
                                    None => {
                                        return Err(
                                            DataTypeReaderError::EnvironmentVariableNotSet(
                                                stringify!(#env_offset).to_string(),
                                                stringify!(#struct_name).to_string()));
                                    }
                                };
                                let type_size = std::mem::size_of::< #generic_field_type >();
                                let size_modulo = size % type_size;
                                if size_modulo != 0 {
                                    return Err(DataTypeReaderError::DirectoryEntrySize(
                                        size,
                                        type_size,
                                        size_modulo));
                                }
                                let capacity = size / type_size;
                                let mut #field_identifier: #field_type = Vec::with_capacity(capacity);

                                datareader.set_position(offset as u64);
                                datareader. #read_type (&mut #field_identifier)?;
                            }
                        }
                    };

                    let qfi = current_field.identifier;
                    let id= current_field.identifier_formated;
                    let s = quote! {
                        #qfi : #id,
                        };
                    (read, s)
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
    let generic_dt = if dtsi.generic { "GENERIC" } else { "" };

    let mut datatype = match tag_value.get("prefix") {
        Some(p) => {
            let prefix = if let syn::Lit::Str(p) = p {
                
                p.value()
            } else {
                panic!("datatypereader attribute prefix's value needs to be a String");
            };
            let i = format_ident!(
                "{}{}{}",
                prefix.to_uppercase(),
                struct_name.to_string().to_uppercase(),
                generic_dt
            );
            quote! { DataType::#i}
        }
        None => {
            let i = format_ident!("{}{}", struct_name.to_string().to_uppercase(), generic_dt);
            quote! { DataType::#i}
        }
    };

    if !dtsi.generic {
        datatype = quote! { #datatype(self.clone()) };
    }

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
                    #datatype
                }
            }
        }
    };

    // Return the generated implementation as a TokenStream
    gen.into()
}
