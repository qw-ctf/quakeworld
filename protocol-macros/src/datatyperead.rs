use std::rc::Rc;

mod argument_parser;

use argument_parser::*;

use syn::{
    braced, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, AttributeArgs, Expr, PathSegment, Token,
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

#[derive(Debug, Default, Clone)]
pub struct StructAttr {
    pub prefix: Option<syn::LitStr>, // prefix for the struct
    pub datatype: StructDataType,    // datatype overwrite for the struct
    pub ommit_trait: OmmitableTrait,
}

impl ParserApplyFunction for StructAttr {
    fn apply_parsed_attribute(&mut self, attribute: &AttributeParse) -> syn::Result<()> {
        match attribute.name.as_str() {
            "prefix" => prefix_apply_to_struct(attribute, self)?,
            "datatype" => datatype_apply_to_struct(attribute, self)?,
            "ommit_trait" => ommit_trait_apply_to_struct(attribute, self)?,
            _ => {
                panic!("not yet implemented!")
            }
        }
        Ok(())
    }
}

macro_rules! attribute_value_error {
    ($input:ident, $type_name:ident) => {
        if ! $input .peek(Token![=]) {
        return Err(syn::Error::new(
        $type_name .span(),
        format!("`{}` attribute does need a value", $type_name),
        ));
        }
        let _: Option<Token![=]> = $input .parse()?;
    }
}

#[derive(Default, PartialEq, Eq)]
enum OffsetParsed {
    #[default]
    None,
    Auto,
    Int(syn::LitInt),
    Str(syn::LitStr),
}

impl From<&SizeOffset> for OffsetParsed {
    fn from(value: &SizeOffset) -> Self {
        match value {
            SizeOffset::None
            | SizeOffset::SizeInt(_)
            | SizeOffset::SizeStr(_)
            | SizeOffset::SizeAuto => OffsetParsed::None,
            SizeOffset::OffsetInt(lit_int) => OffsetParsed::Int(lit_int.clone()),
            SizeOffset::OffsetStr(lit_str) => OffsetParsed::Str(lit_str.clone()),
            SizeOffset::SizeOffsetStrStr(lit_str, _) | SizeOffset::SizeOffsetStrInt(lit_str, _) => {
                OffsetParsed::Str(lit_str.clone())
            }
            SizeOffset::SizeOffsetIntStr(lit_int, _) | SizeOffset::SizeOffsetIntInt(lit_int, _) => {
                OffsetParsed::Int(lit_int.clone())
            }
            SizeOffset::SizeOffsetStr(lit_str) => {
                let name_offset = format!("{}_offset", lit_str.value());
                let name_offset = syn::LitStr::new(&name_offset, lit_str.span());
                OffsetParsed::Str(name_offset)
            }
            SizeOffset::OffsetAuto | SizeOffset::SizeOffsetAuto => OffsetParsed::Auto,
        }
    }
}

#[derive(Default, PartialEq, Eq)]
enum SizeParsed {
    #[default]
    None,
    Auto,
    Int(syn::LitInt),
    Str(syn::LitStr),
}

impl From<&SizeOffset> for SizeParsed {
    fn from(value: &SizeOffset) -> Self {
        match value {
            SizeOffset::None
            | SizeOffset::OffsetInt(_)
            | SizeOffset::OffsetStr(_)
            | SizeOffset::OffsetAuto => SizeParsed::None,
            SizeOffset::SizeInt(lit_int) => SizeParsed::Int(lit_int.clone()),
            SizeOffset::SizeStr(lit_str) => SizeParsed::Str(lit_str.clone()),
            SizeOffset::SizeOffsetStrStr(_, lit_str) | SizeOffset::SizeOffsetIntStr(_, lit_str) => {
                SizeParsed::Str(lit_str.clone())
            }
            SizeOffset::SizeOffsetIntInt(_, lit_int) | SizeOffset::SizeOffsetStrInt(_, lit_int) => {
                SizeParsed::Int(lit_int.clone())
            }
            SizeOffset::SizeOffsetStr(lit_str) => {
                let name_size = format!("{}_size", lit_str.value());
                let name_size = syn::LitStr::new(&name_size, lit_str.span());
                SizeParsed::Str(name_size)
            }
            SizeOffset::SizeAuto | SizeOffset::SizeOffsetAuto => SizeParsed::Auto,
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Debug)]
enum SizeRecalc {
    #[default]
    None,
    ModuloSelfEnvironment,
}

#[derive(Default)]
struct FieldAttributesParsed {
    pub treat_as_string: bool,
    pub size: SizeParsed,
    pub size_recalc: SizeRecalc,
    pub offset: OffsetParsed,
    pub environment: Environment,
}

fn parse_attribute_type_blank(input: &mut ParseStream, name: syn::Ident) -> syn::Result<bool> {
    // println!(
    //     "we try and parse a blank {} {} {}",
    //     input.peek(Token![,]),
    //     !input.peek(token::Paren),
    //     input.peek(token::Eq)
    // );
    if input.peek(token::Eq) || input.peek(token::Paren) {
        // println!("{:?}", input);
        // println!("{}", name);
        return Err(syn::Error::new(
            name.span(),
            format!("`{}` does not take values", name),
        ));
    }
    // println!("we parsed blank for: {}", name);
    Ok(true)
}

fn parse_attribute_type_vector(
    input: &mut ParseStream,
    allowed: Vec<AttributeTypeAllowed>,
    name: syn::Ident,
) -> syn::Result<bool> {
    Ok(true)
}

fn parse_attribute_type_single(
    input: &mut ParseStream,
    allowed: Vec<AttributeTypeAllowed>,
    name: syn::Ident,
) -> syn::Result<bool> {
    Ok(true)
}

impl AttributeType {
    fn parse(&self, input: &mut ParseStream, name: syn::Ident) -> syn::Result<bool> {
        println!("we parse this in here");
        let t = match self {
            AttributeType::Blank => parse_attribute_type_blank(input, name)?,
            AttributeType::Single(attribute_type_alloweds) => {
                parse_attribute_type_single(input, (*attribute_type_alloweds).clone(), name)?
            }
            AttributeType::Vector(attribute_type_alloweds) => {
                parse_attribute_type_vector(input, (*attribute_type_alloweds).clone(), name)?
            }
        };
        Ok(t)
    }
}

macro_rules! generate_attribute_options {
    ($(($key:expr, ($($inner:expr),*))),* $(,)?) => {
        {

            let mut results: Vec<AttributeParse> = Vec::new();
            $(
                let mut v = AttributeParse{ name: $key.into(), types: vec![]};
                $(
                    v.types.push($inner);
                )*
                results.push(v);
            )*
        results
        }
    };
}

macro_rules! return_syn_error {
    ($name:expr, $errortype:expr ) => {
        return Err(syn::Error::new(
            $name.span(),
            format!(
                "`{}` attribute does not support the `{}` value",
                $name, $errortype
            ),
        ))
    };
}

fn prefix_apply_to_struct(
    attribute: &AttributeParse,
    struct_attributes: &mut StructAttr,
) -> syn::Result<()> {
    // let attr_ident = attribute.name_ident.clone().unwrap();
    match &attribute.parsed_value {
        AttributeTypeParsed::None => {
            return Ok(());
        }
        AttributeTypeParsed::Blank => return_syn_error!(attribute.name_ident, "Blank"),
        AttributeTypeParsed::Single(attribute_value) => match attribute_value {
            AttributeValue::Int(lit_int) => return_syn_error!(attribute.name_ident, "Single Int"),
            AttributeValue::Str(lit_str) => {
                struct_attributes.prefix = Some(lit_str.clone());
            }
            AttributeValue::Ident(ident) => return_syn_error!(attribute.name_ident, "Single Type"),
        },
        AttributeTypeParsed::Vector(attribute_values) => {
            return_syn_error!(attribute.name_ident, "Vector")
        }
    }
    Ok(())
}

fn datatype_apply_to_struct(
    attribute: &AttributeParse,
    struct_attributes: &mut StructAttr,
) -> syn::Result<()> {
    // let attr_ident = attribute.name_ident.clone().unwrap();
    match &attribute.parsed_value {
        AttributeTypeParsed::None => {
            // return_syn_error!(attribute.name_ident, "None")
            return Ok(());
        }
        AttributeTypeParsed::Blank => return_syn_error!(attribute.name_ident, "Blank"),
        AttributeTypeParsed::Single(attribute_value) => match attribute_value {
            AttributeValue::Int(lit_int) => return_syn_error!(attribute.name_ident, "Single Int"),
            AttributeValue::Str(lit_str) => {
                return_syn_error!(attribute.name_ident, "Single Type")
            }
            AttributeValue::Ident(ident) => {
                // TODO: apply this
                // struct_attributes.datatype = ident.clone()
                struct_attributes.datatype = StructDataType::Ident(ident.clone());
            }
        },
        AttributeTypeParsed::Vector(attribute_values) => {
            return_syn_error!(attribute.name_ident, "Vector")
        }
    }
    Ok(())
}

fn ommit_trait_apply_to_struct(
    attribute: &AttributeParse,
    struct_attributes: &mut StructAttr,
) -> syn::Result<()> {
    match &attribute.parsed_value {
        AttributeTypeParsed::None => {
            return Ok(());
            // return_syn_error!(attribute.name_ident, "None")
        }
        AttributeTypeParsed::Blank => return_syn_error!(attribute.name_ident, "Blank"),
        AttributeTypeParsed::Single(attribute_value) => match attribute_value {
            AttributeValue::Int(lit_int) => return_syn_error!(attribute.name_ident, "Single Int"),
            AttributeValue::Str(lit_str) => {
                return_syn_error!(attribute.name_ident, "Single Type")
            }
            AttributeValue::Ident(ident) => {
                // TODO: apply this
                // struct_attributes.datatype = ident.clone()
                // struct_attributes.datatype = StructDataType::Ident(ident.clone());
                let ident = ident.to_string();
                match ident.as_str() {
                    "DataTypeSize" => struct_attributes.ommit_trait.size = true,
                    "DataTypeRead" => struct_attributes.ommit_trait.read = true,
                    _ => {
                        return_syn_error!(attribute.name_ident, ident);
                    }
                }
            }
        },
        AttributeTypeParsed::Vector(attribute_values) => {
            return_syn_error!(attribute.name_ident, "Vector")
        }
    }
    Ok(())
}

impl Parse for StructAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut struct_attributes = StructAttr::default();

        let mut attribute_parser = AttributeParser::default();

        // prefix
        let mut a = AttributeParse::new("prefix");
        // only allows a single Str
        a.add_type(AttributeType::Single(vec![AttributeTypeAllowed::Str]));
        // a.apply_to_struct = Some(prefix_apply_to_struct);
        attribute_parser.add_attribute(a);

        // datatype
        let mut a = AttributeParse::new("datatype");
        // only allows a single Str
        a.add_type(AttributeType::Single(vec![AttributeTypeAllowed::Ident]));
        // a.apply_to_struct = Some(datatype_apply_to_struct);
        attribute_parser.add_attribute(a);

        // ommit_trait
        let mut a = AttributeParse::new("ommit_trait");
        // only allows a single Str
        a.add_type(AttributeType::Single(vec![AttributeTypeAllowed::Ident]));
        // a.apply_to_struct = Some(ommit_trait_apply_to_struct);
        attribute_parser.add_attribute(a);

        // ommit_func
        // let mut a = AttributeParse::new("ommit_func");
        // // only allows a single Str
        // a.add_type(AttributeType::Blank);
        // attribute_parser.add_attribute(a);

        let mut i = input.clone();
        attribute_parser.parse_attributes(&mut i)?;
        // attribute_parser.apply_to_struct(&mut struct_attributes)?;
        // match attribute_parser.apply_to_struct(&mut struct_attributes) {
        //     Ok(_) => {}
        //     Err(e) => {
        //         println!("we crash here? {}", e);
        //         return Err(e);
        //     }
        // };

        for attr in &attribute_parser.attributes {
            struct_attributes.apply_parsed_attribute(attr)?;
        }

        if false {
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
                            return Err(syn::Error::new(
                                type_name.span(),
                                format!("`{}` attribute does not support the value", type_name),
                            ));
                        } else {
                            return Err(syn::Error::new(
                                type_name.span(),
                                format!("`{}` attribute does not support the value", type_name),
                            ));
                        }
                    }
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

                            return Err(syn::Error::new(
                                type_name.span(),
                                format!("`{}` attribute does not support the value", type_name),
                            ));
                        } else {
                            return Err(syn::Error::new(
                                type_name.span(),
                                format!("`{}` attribute does not support the value", type_name),
                            ));
                        }
                    }
                    "ommit_trait" => {
                        attribute_value_error!(input, type_name);
                        if !input.peek(syn::Ident) {
                            return Err(syn::Error::new(
                                type_name.span(),
                                format!("`{}` attribute value needs to be Ident", type_name),
                            ));
                        }

                        let s: syn::Ident = input.parse()?;

                        match s.to_string().as_str() {
                            "DataTypeSize" => {
                                struct_attributes.ommit_trait.size = true;
                            }
                            "DataTypeRead" => {
                                struct_attributes.ommit_trait.read = true;
                            }
                            _ => {
                                return Err(syn::Error::new(
                                    type_name.span(),
                                    format!(
                                        "`{}` attribute value `{}` not supported",
                                        type_name, s
                                    ),
                                ));
                            }
                        }
                        continue;
                    }
                    "replace" => {
                        attribute_value_error!(input, type_name);
                    }
                    _ => {
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` attribute not supported", type_name),
                        ))
                    }
                }
            }
        }
        println!(
            "-------> the struct attr we return: {:?}",
            struct_attributes
        );
        Ok(struct_attributes)
    }
}

#[derive(Debug, Clone, Default)]
enum SizeOffset {
    #[default]
    None,
    SizeInt(syn::LitInt),
    SizeStr(syn::LitStr),
    SizeAuto,
    OffsetInt(syn::LitInt),
    OffsetStr(syn::LitStr),
    OffsetAuto,
    SizeOffsetStrStr(syn::LitStr, syn::LitStr),
    SizeOffsetStrInt(syn::LitStr, syn::LitInt),
    SizeOffsetIntStr(syn::LitInt, syn::LitStr),
    SizeOffsetIntInt(syn::LitInt, syn::LitInt),
    SizeOffsetStr(syn::LitStr),
    SizeOffsetAuto,
}

#[derive(Debug, Clone, Default)]
enum Environment {
    #[default]
    None,
    Auto,
    String(syn::LitStr),
}

#[derive(Debug, Clone)]
enum FieldAttribute {
    SizeOffset(SizeOffset),
    EnvironmentSet(Environment),
    String, // Parse a Vec<u8> as a string
    SizeRecalc(SizeRecalc),
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
                            field_attributes
                                .push(FieldAttribute::EnvironmentSet(Environment::String(s)));
                            continue;
                        }
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` attribute does not support the value", type_name),
                        ));
                    } else {
                        field_attributes.push(FieldAttribute::EnvironmentSet(Environment::Auto));
                        continue;
                    }
                }
                "string" => {
                    field_attributes.push(FieldAttribute::String);
                }
                "size_from" => {
                    if input.peek(Token![=]) {
                        let _: Option<Token![=]> = input.parse()?;
                        if input.peek(syn::LitInt) {
                            let i: syn::LitInt = input.parse()?;
                            field_attributes
                                .push(FieldAttribute::SizeOffset(SizeOffset::SizeInt(i)));
                            continue;
                        }
                        if input.peek(syn::LitStr) {
                            let s: syn::LitStr = input.parse()?;
                            field_attributes
                                .push(FieldAttribute::SizeOffset(SizeOffset::SizeStr(s)));
                            continue;
                        }

                        if input.peek(syn::Ident) {
                            let i: syn::Ident = input.parse()?;
                            // field_attributes.push(
                            //     FieldAttribute::SizeOffset(
                            //         SizeOffset::SizeStr(s)));
                            continue;
                        }
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` attribute does not support the value", type_name),
                        ));
                    } else if input.is_empty() || input.peek(Token![,]) {
                        field_attributes.push(FieldAttribute::SizeOffset(SizeOffset::SizeAuto));
                    } else {
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` attribute needs a = value", type_name),
                        ));
                    }
                }
                "offset_from" => {
                    if input.peek(Token![=]) {
                        let _: Option<Token![=]> = input.parse()?;
                        if input.peek(syn::LitInt) {
                            let i: syn::LitInt = input.parse()?;
                            field_attributes
                                .push(FieldAttribute::SizeOffset(SizeOffset::OffsetInt(i)));
                            continue;
                        }
                        if input.peek(syn::LitStr) {
                            let s: syn::LitStr = input.parse()?;
                            field_attributes
                                .push(FieldAttribute::SizeOffset(SizeOffset::OffsetStr(s)));
                            continue;
                        }
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` attribute does not support the value", type_name),
                        ));
                    } else if input.is_empty() || input.peek(Token![,]) {
                        field_attributes.push(FieldAttribute::SizeOffset(SizeOffset::OffsetAuto));
                    } else {
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` attribute needs a = value", type_name),
                        ));
                    }
                }
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
                                    field_attributes.push(FieldAttribute::SizeOffset(
                                        SizeOffset::SizeOffsetIntInt(
                                            first_field_int,
                                            second_field_int,
                                        ),
                                    ));
                                    continue;
                                }
                                if content.peek(syn::LitStr) {
                                    let second_field_str: syn::LitStr = content.parse()?;
                                    field_attributes.push(FieldAttribute::SizeOffset(
                                        SizeOffset::SizeOffsetIntStr(
                                            first_field_int,
                                            second_field_str,
                                        ),
                                    ));
                                    continue;
                                }
                                return Err(syn::Error::new(
                                    type_name.span(),
                                    format!(
                                        "`{}` attribute first value supplied is wrong",
                                        type_name
                                    ),
                                ));
                            } else if content.peek(syn::LitStr) {
                                let first_field_str: syn::LitStr = content.parse()?;
                                if content.peek(syn::LitInt) {
                                    let second_field_int: syn::LitInt = content.parse()?;
                                    field_attributes.push(FieldAttribute::SizeOffset(
                                        SizeOffset::SizeOffsetStrInt(
                                            first_field_str,
                                            second_field_int,
                                        ),
                                    ));
                                    continue;
                                }
                                if content.peek(syn::LitStr) {
                                    let second_field_str: syn::LitStr = content.parse()?;
                                    field_attributes.push(FieldAttribute::SizeOffset(
                                        SizeOffset::SizeOffsetStrStr(
                                            first_field_str,
                                            second_field_str,
                                        ),
                                    ));
                                    continue;
                                }
                                return Err(syn::Error::new(
                                    type_name.span(),
                                    format!(
                                        "`{}` attribute second value supplied is wrong",
                                        type_name
                                    ),
                                ));
                            }
                            return Err(syn::Error::new(
                                type_name.span(),
                                format!(
                                    "`{}` attribute does not support the value supplied",
                                    type_name
                                ),
                            ));
                        } else if input.peek(syn::LitInt) {
                            return Err(syn::Error::new(
                                type_name.span(),
                                format!(
                                    "`{}` attribute does not support the value supplied",
                                    type_name
                                ),
                            ));
                        } else if input.peek(syn::LitStr) {
                            let s = input.parse()?;
                            field_attributes
                                .push(FieldAttribute::SizeOffset(SizeOffset::SizeOffsetStr(s)));
                            continue;
                        }
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!(
                                "`{}` attribute does not support the value supplied",
                                type_name
                            ),
                        ));
                    } else if input.peek(Token![,]) || input.is_empty() {
                        field_attributes
                            .push(FieldAttribute::SizeOffset(SizeOffset::SizeOffsetAuto));
                        continue;
                    } else {
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` attribute needs a = value", type_name),
                        ));
                    }
                }
                "size" => {
                    if !input.peek(Token![=]) {
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` attribute needs a = value", type_name),
                        ));
                    }
                    let _: Option<Token![=]> = input.parse()?;
                    if !input.peek(syn::Ident) {
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` only accepts literals", type_name),
                        ));
                    }
                    let ident: syn::Ident = input.parse()?;
                    let s = ident.to_string();
                    if let "modulo_self_environment" = s.as_str() {
                        field_attributes.push(FieldAttribute::SizeRecalc(
                            SizeRecalc::ModuloSelfEnvironment,
                        ));
                        continue;
                    } else {
                        return Err(syn::Error::new(
                            type_name.span(),
                            format!("`{}` not a valid `{}` option", s, type_name),
                        ));
                    }
                }
                _ => {
                    return Err(syn::Error::new(
                        type_name.span(),
                        format!("`{}` attribute not supported", type_name),
                    ))
                }
            }
        }
        Ok(FieldAttr {
            attributes: field_attributes,
        })
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
            }
            .into()
        }
        syn::Data::Union(data_union) => {
            return quote_spanned! {
                data_union.union_token.span =>
                compile_error!("DataTypeReader can only be used on structs not unions");
            }
            .into()
        }
    };

    let mut struct_attrib = StructAttr::default();
    for attr in &input.attrs {
        if !attr.path.is_ident("datatyperead") {
            continue;
        }
        // println!("{:?}", attr);

        let attr_span = attr.span();
        let a = attr.parse_args_with(Punctuated::<StructAttr, Token![,]>::parse_terminated);
        if let Ok(sa) = a {
            {
                struct_attrib = sa[0].clone();
                println!("\tfindme ---> {:?}\n\t --> {}", struct_attrib, input.ident);
            }
        };
    }

    // println!("findme ---> {:?}", struct_attrib);

    if input.ident.clone() == "SizedVectorNameTest" {
        println!(
            "we are here right: {:?} {}",
            input.generics.params.first(),
            input.generics.params.first().is_some()
        );
    }

    let is_generic = input.generics.params.first().is_some();

    let mut struct_information = StructInformation {
        identifier: input.ident.clone(),
        generics: input.generics,
        is_generic,
        fields: vec![],
        attributes: struct_attrib,
    };
    println!("{}", input.ident);
    // println!("{}", input.generics.params.to_token_stream());
    println!("{:?}", struct_information);
    for field in data_struct.fields.iter() {
        // println!("{:?}", field.ty);
        // match &field.ty {
        //     Type::Array(type_array) => {}
        //     Type::BareFn(type_bare_fn) => {}
        //     Type::Group(type_group) => {}
        //     Type::ImplTrait(type_impl_trait) => {}
        //     Type::Infer(type_infer) => {}
        //     Type::Macro(type_macro) => {}
        //     Type::Never(type_never) => {}
        //     Type::Paren(type_paren) => {}
        //     Type::Path(type_path) => {
        //         // println!("{:?}", type_path);
        //     }
        //     Type::Ptr(type_ptr) => {}
        //     Type::Reference(type_reference) => {}
        //     Type::Slice(type_slice) => {}
        //     Type::TraitObject(type_trait_object) => {}
        //     Type::Tuple(type_tuple) => {}
        //     Type::Verbatim(token_stream) => {}
        //     _ => {}
        // };
        let field_span = field.span();
        let field_ident =
            match &field.ident {
                Some(f) => f.clone(),
                None => return quote_spanned! {
                    field_span =>
                    compile_error!("DataTypeReader can only be used on structs with named fields");
                }
                .into(),
            };

        let field_ty = field.ty.clone();

        let mut field_attributes: Vec<FieldAttribute> = vec![];

        for attr in &field.attrs {
            if !attr.path.is_ident("datatyperead") {
                continue;
            }
            let attr_span = attr.span();
            let a = attr.parse_args_with(Punctuated::<FieldAttr, Token![,]>::parse_terminated);
            match a {
                Ok(a) => {
                    for attribute in a[0].attributes.clone() {
                        field_attributes.push(attribute);
                    }
                }
                Err(e) => {
                    let b = format!("{}", e);
                    return quote_spanned! {
                        attr_span =>
                        compile_error!(#b);
                    }
                    .into();
                }
            };
        }
        struct_information.fields.push(FieldInformation {
            identifier: field_ident,
            ty: field_ty,
            attributes: field_attributes,
        });
    }
    // println!("we here?: {:?}", struct_information);

    let prefix = match &struct_information.attributes.prefix {
        Some(e) => e.value().clone().to_uppercase(),
        None => "".to_string(),
    };
    let struct_name = struct_information
        .identifier
        .clone()
        .to_string()
        .to_uppercase();
    // let (_, generic_types, _) = struct_information.generics.split_for_impl();
    let datatype = match struct_information.generics.params.first() {
        Some(_) => format_ident!("{}{}GENERIC", prefix, struct_name),
        None => format_ident!("{}{}", prefix, struct_name),
    };

    let mut field_creations: Vec<_> = vec![];
    let mut field_assignments: Vec<_> = vec![];
    let mut field_errors: Vec<_> = vec![];
    let mut field_sizes: Vec<_> = vec![];

    struct_information.fields.iter().for_each(|f| {
        let mut field_creation: Vec<_> = vec![];
        let mut field_size: Vec<_> = vec![];
        let mut field_assignment: Vec<_> = vec![];
        let mut field_error: Vec<_> = vec![];

        // println!("parsing field: {}", f.identifier);

        // yes i am aware
        let mut fap = FieldAttributesParsed::default();

        // let mut field_attributes: Vec<TokenStream> = vec![];
        for attribute in &f.attributes {
            match attribute {
                FieldAttribute::SizeOffset(size_offset) => {
                    let s: SizeParsed = size_offset.into();
                    if fap.size != SizeParsed::None && s != SizeParsed::None {
                        field_error.push(quote_spanned! {
                            f.identifier.span() =>
                            compile_error!("Size set twice");
                        });
                    } else if fap.size == SizeParsed::None {
                        fap.size = s;
                    }

                    let o: OffsetParsed = size_offset.into();
                    if fap.offset != OffsetParsed::None && o != OffsetParsed::None {
                        field_error.push(quote_spanned! {
                            f.identifier.span() =>
                            compile_error!("Offset set twice");
                        });
                    } else if fap.offset == OffsetParsed::None {
                        fap.offset = o;
                    }
                }
                FieldAttribute::EnvironmentSet(environment) => {
                    fap.environment = environment.clone()
                }
                FieldAttribute::String => fap.treat_as_string = true,
                FieldAttribute::SizeRecalc(size_recalc) => {
                    fap.size_recalc = size_recalc.clone();
                }
            }
        }
        let field_name = f.identifier.clone();
        let field_identifier = format_ident!("{}_identifier", f.identifier.clone());
        let field_type = f.ty.clone();
        // let newline = format_ident!("\n");
        // let newline = Char::from_u8(10).unwrao();

        let read_exect_type = match fap.treat_as_string {
            true => quote! { read_exact_generic_string },
            false => quote! { read_exact_generic_v2 },
        };

        let field_environment = match fap.environment {
            Environment::None => quote! {},
            Environment::Auto => quote! {
                #field_identifier . environment( datareader , stringify!(#field_name));
            },
            Environment::String(lit_str) => quote! {
                #field_identifier . environment( datareader , #lit_str);
            },
        };

        let mut field_offset_after = quote! {};
        let field_offset = match fap.offset {
            OffsetParsed::None => quote! {},
            OffsetParsed::Int(lit_int) => {
                field_offset_after = quote! {
                    datareader.set_position(old_offset);
                };
                quote! {
                    let old_offset = datareader.position();
                    datareader.set_position(#lit_int);
                }
            }
            OffsetParsed::Str(lit_str) => {
                field_offset_after = quote! {
                    datareader.set_position(old_offset);
                };
                quote! {
                    let old_offset = datareader.position();
                    let current_field_offset: u64 = datareader.get_env_error(#lit_str)?.into();
                    datareader.set_position(current_field_offset);
                }
            }
            OffsetParsed::Auto => {
                field_offset_after = quote! {
                    datareader.set_position(old_offset);
                };
                let f_n = format!("{}_offset", field_name);
                quote! {
                    let old_offset = datareader.position();
                    let current_field_offset: u64 = datareader.get_env_error(#f_n)?.into();
                    datareader.set_position(current_field_offset);
                }
            }
        };

        let field_size_recalc = match fap.size_recalc {
            SizeRecalc::None => quote!(),
            SizeRecalc::ModuloSelfEnvironment => {
                quote! {
                    // let modulo_type_size = std::mem::size_of::< #field_type >();
                    let modulo_type_size = < #field_type >::datatype_size();
                    let modulo_original_size = size_from_environment;
                    let modulo_remainder = size_from_environment % modulo_type_size;
                    if modulo_remainder != 0 {
                        return Err(DataTypeReaderError::DirectoryEntrySize(
                            modulo_original_size,
                            modulo_type_size,
                            modulo_remainder));
                    }
                    let size_from_environment = size_from_environment / modulo_type_size;
                }
            }
        };

        // Generating field reading
        let fc = match fap.size {
            SizeParsed::None => quote! {
                #field_offset
                let #field_identifier = <#field_type as DataTypeRead>::read(datareader)?;
                #field_offset_after
                #field_environment
            },
            SizeParsed::Int(lit_int) => quote! {
                let size_from_environment: usize = #lit_int;
                let mut #field_identifier: #field_type = Vec::with_capacity(size_from_environment);
                #field_offset
                datareader. #read_exect_type (&mut #field_identifier)?;
                #field_offset_after
                #field_environment
            },
            SizeParsed::Str(lit_str) => quote! {
                let size_from_environment: usize = datareader.get_env_error(#lit_str)? .into();
                let mut #field_identifier: #field_type = Vec::with_capacity(size_from_environment);
                #field_size_recalc
                #field_offset
                datareader.read_exact_generic_v2(&mut #field_identifier)?;
                #field_offset_after
                #field_environment
            },
            SizeParsed::Auto => {
                let f_n = format!("{}_size", field_name);
                quote! {
                let size_from_environment: usize = datareader.get_env_error(#f_n)? .into();
                #field_size_recalc
                let mut #field_identifier: #field_type = Vec::with_capacity(size_from_environment);
                #field_offset
                datareader.read_exact_generic_v2(&mut #field_identifier)?;
                #field_offset_after
                #field_environment }
            }
        };

        // let fc = match f.identifier { quote!{
        //     // let #field_identifier: #field_type = datareader.read()?;
        //     let #field_identifier = <#field_type as DataTypeRead>::read(datareader)?;
        // }.to_token_stream();}

        let fa = quote! {
            #field_name: #field_identifier,
        }
        .to_token_stream();
        let fs = quote! {
            size = size + < #field_type as DataTypeSize>::datatype_size();
        };
        field_creation.push(fc);
        field_assignment.push(fa);
        field_size.push(fs);

        for f in field_size.into_iter() {
            field_sizes.push(f);
        }

        for f in field_error.into_iter() {
            field_errors.push(f);
        }

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
    let (struct_impl_generics, struct_type_generics, struct_where_clause) =
        struct_information.generics.split_for_impl();

    let si_identifier = {
        let ident = struct_information.identifier.clone();
        quote! { #ident }
    };

    let struct_name = match struct_information.attributes.prefix.clone() {
        Some(p) => {
            // let prefix = format_ident!("{}", p.value());
            quote! { #prefix::#p }
        }
        None => quote! {#si_identifier},
    };

    let has_data = match struct_information.is_generic {
        false => quote! { (self.clone())},
        true => quote! {},
    };

    let datatype_overwrite = match struct_information.attributes.datatype {
        StructDataType::None => quote! { DataType :: #datatype #has_data },
        StructDataType::String(ref ident) => quote! { DataType :: #ident },
        StructDataType::Ident(ref ident) => {
            let ident = ident.clone();
            quote! { DataType :: #ident }
        } // None => quote!{ DataType :: #datatype #has_data },
    };

    let size_trait = match struct_information.attributes.ommit_trait.size {
        true => quote! {},
        false => {
            quote! {
                impl #struct_impl_generics DataTypeSize for #si_identifier #struct_type_generics #struct_where_clause {
                    fn datatype_size() -> usize {
                        let mut size: usize = 0;
                        #(#field_sizes)*
                        size
                    }
                }
            }
        }
    };

    let read_trait = quote! {
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
    let read_trait = match struct_information.attributes.ommit_trait.read {
        true => quote! {},
        false => read_trait,
    };

    let gen = quote! {
        #read_trait
        #size_trait
    };

    if struct_name.to_string() == "Vector3" {
        // println!("{} -> {:?}", struct_name, struct_information);
        // panic!();
    }
    gen.into()
}
