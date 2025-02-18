use std::fmt::Debug;
use std::rc::Rc;

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

#[derive(Debug, Clone)]
pub enum AttributeTypeAllowed {
    Int,
    Str,
    Ident,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    Int(syn::LitInt),
    Str(syn::LitStr),
    Ident(syn::Ident),
}

impl From<syn::LitInt> for AttributeValue {
    fn from(value: syn::LitInt) -> Self {
        return AttributeValue::Int(value.clone());
    }
}

impl From<syn::LitStr> for AttributeValue {
    fn from(value: syn::LitStr) -> Self {
        return AttributeValue::Str(value.clone());
    }
}

impl From<syn::Ident> for AttributeValue {
    fn from(value: syn::Ident) -> Self {
        return AttributeValue::Ident(value.clone());
    }
}

#[derive(Debug, Default, Clone)]
pub struct OmmitableTrait {
    pub size: bool,
    pub read: bool,
}

#[derive(Debug, Clone, Default)]
pub enum StructDataType {
    #[default]
    None,
    String(syn::LitStr),
    Ident(syn::Ident),
}

#[derive(Debug, Clone)]
pub enum AttributeType {
    Blank,
    Single(Vec<AttributeTypeAllowed>),
    Vector(Vec<AttributeTypeAllowed>),
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum AttributeTypeParsed {
    #[default]
    None,
    Blank,
    Single(AttributeValue),
    Vector(Vec<AttributeValue>),
}

#[derive(Debug)]
pub struct AttributeParse {
    pub name: String,
    pub name_ident: syn::Ident,
    pub blank: bool,
    pub parsed_value: AttributeTypeParsed,
    pub single: Option<Vec<AttributeTypeAllowed>>,
    pub vector: Option<Vec<AttributeTypeAllowed>>,
    pub types: Vec<AttributeType>,
    // pub apply_to_struct: Option<fn(&AttributeParse, &mut StructAttr) -> syn::Result<()>>,
}

impl Default for AttributeParse {
    fn default() -> Self {
        // let name_ident: syn::Ident = Ident::new("default", Span::call_site()).into();
        let name_ident: syn::Ident = format_ident!("default");
        Self {
            name: Default::default(),
            name_ident,
            blank: Default::default(),
            parsed_value: Default::default(),
            single: Default::default(),
            vector: Default::default(),
            types: Default::default(),
            // apply_to_struct: Default::default(),
        }
    }
}

impl AttributeParse {
    pub fn new(name: impl Into<String>) -> Self {
        return AttributeParse {
            name: name.into(),
            ..Default::default()
        };
    }

    pub fn add_type(&mut self, add_value: AttributeType) {
        match add_value.clone() {
            AttributeType::Blank => {
                if self.blank {
                    panic!("cant set Blank argument twice")
                }
                self.blank = true;
            }
            AttributeType::Single(attribute_type_alloweds) => {
                if self.single.is_some() {
                    panic!("cant set Single argument twice")
                }
                self.single = Some(attribute_type_alloweds);
            }
            AttributeType::Vector(attribute_type_alloweds) => {
                if self.vector.is_some() {
                    panic!("cant set Vector argument twice")
                }
                self.vector = Some(attribute_type_alloweds);
            }
        }
        // self.types.push(add_value);
    }

    pub fn parse(&mut self, input: &ParseStream, name: syn::Ident) -> syn::Result<bool> {
        // check if its an value
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            // we cant do those so we fail
            if !self.single.is_some() && !self.vector.is_some() {
                return Err(syn::Error::new(
                    name.span(),
                    format!("`{}` does not support Single or Vector", name),
                ));
            }

            if !input.peek(token::Paren) {
                // handle single
                let single = match &self.single {
                    Some(v) => v.clone(),
                    None => {
                        return Err(syn::Error::new(
                            name.span(),
                            format!("`{}` does not support Single", name),
                        ));
                    }
                };

                let mut found = false;
                for ty in single {
                    match ty {
                        AttributeTypeAllowed::Int => {
                            if input.peek(syn::LitInt) {
                                let i: syn::LitInt = input.parse()?;
                                found = true;
                                self.parsed_value = AttributeTypeParsed::Single(i.into());
                            }
                        }
                        AttributeTypeAllowed::Str => {
                            if input.peek(syn::LitStr) {
                                let s: syn::LitStr = input.parse()?;
                                found = true;
                                self.parsed_value = AttributeTypeParsed::Single(s.into());
                            }
                        }
                        AttributeTypeAllowed::Ident => {
                            if input.peek(syn::Ident) {
                                let t: syn::Ident = input.parse()?;
                                found = true;
                                self.parsed_value = AttributeTypeParsed::Single(t.into());
                            }
                        }
                    };
                    if found {
                        return Ok(true);
                    }
                }
            }

            // handle Vector
            let tya = match &self.vector {
                Some(v) => v.clone(),
                None => {
                    return Err(syn::Error::new(
                        name.span(),
                        format!("`{}` does not support Vector", name),
                    ));
                }
            };

            let content;
            let _ = parenthesized!(content in input);

            loop {
                let mut parsed_values: Vec<AttributeValue> = vec![];
                for ty in &tya {
                    let mut found = false;
                    match ty {
                        AttributeTypeAllowed::Int => {
                            if content.peek(syn::LitInt) {
                                let i: syn::LitInt = content.parse()?;
                                found = true;
                                parsed_values.push(i.into());
                            }
                        }
                        AttributeTypeAllowed::Str => {
                            if content.peek(syn::LitStr) {
                                let s: syn::LitStr = content.parse()?;
                                found = true;
                                parsed_values.push(s.into());
                            }
                        }
                        AttributeTypeAllowed::Ident => {
                            if content.peek(syn::Ident) {
                                let t: syn::Ident = content.parse()?;
                                found = true;
                                parsed_values.push(t.into());
                            }
                        }
                    };
                    if !found {
                        return Err(syn::Error::new(
                            name.span(),
                            format!("`{}` does not support that Vector Argument", name),
                        ));
                    }
                }
                if content.is_empty() {
                    break;
                }
            }
        }

        if !self.blank {
            return Err(syn::Error::new(
                name.span(),
                format!("`{}` does not support Blank", name),
            ));
        }

        if self.parsed_value != AttributeTypeParsed::None {
            return Err(syn::Error::new(
                name.span(),
                format!("`{}` was already parsed?", name),
            ));
        }
        self.parsed_value = AttributeTypeParsed::Blank;
        Ok(true)
    }
}

#[derive(Debug, Default)]
pub struct AttributeParser {
    pub attributes: Vec<AttributeParse>,
}

impl AttributeParser {
    pub fn add_attribute(&mut self, attribute: AttributeParse) {
        self.attributes.push(attribute);
    }

    pub fn parse_attributes(&mut self, input: &ParseStream) -> syn::Result<bool> {
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

            if !input.peek(syn::Ident) {
                return Err(syn::Error::new(input.span(), "malformed macro invocation"));
            }
            let ident: syn::Ident = input.parse()?;
            let found = false;

            for attribute in &mut self.attributes {
                let ident_name = ident.clone().to_string();
                let i = ident.clone();
                attribute.name_ident = ident.clone();
                if attribute.name == ident_name {
                    attribute.name_ident = ident.clone();
                    if attribute.parse(input, i)? {
                        break;
                    }
                }
            }
            if found {
                continue;
            }

            if input.is_empty() {
                break;
            }

            // we can have multiple attribute types
            if input.peek(Token![,]) {
                continue;
            }

            return Err(syn::Error::new(
                input.span(),
                format!("`{}` not a valid option/option not implemented", ident),
            ));
        }
        Ok(true)
    }

    // fn apply_to_struct(&mut self, struct_attributes: &mut StructAttr) -> syn::Result<()> {
    //     for attribute in &self.attributes {
    //         match attribute.apply_to_struct {
    //             Some(f) => (f)(attribute, struct_attributes)?,
    //             None => {
    //                 return Err(syn::Error::new(
    //                     attribute.name.span(),
    //                     format!(
    //                         "`{}` has not implemented the apply_to_struct function",
    //                         attribute.name
    //                     ),
    //                 ));
    //             }
    //         };
    //     }
    //     Ok(())
    // }

    fn apply_to(&mut self, to_value: &mut impl ParserApplyFunction) -> syn::Result<()> {
        for attribute in &self.attributes {
            to_value.apply_parsed_attribute(attribute)?;
        }
        Ok(())
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

// these allow us to pass anything
// pub trait ParserApply<T: Debug = Self> {
pub trait ParserApply {
    fn apply_attribute(&self, attribute_parser: &AttributeParse) -> bool;
}

pub trait ParserApplyFunction: Debug {
    fn apply_parsed_attribute(&mut self, attribute: &AttributeParse) -> syn::Result<()>;
}
