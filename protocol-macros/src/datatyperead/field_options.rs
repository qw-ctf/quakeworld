use super::{argument_parser::AttributeParse, Environment, FieldAttr, SizeOffset};

macro_rules! return_syn_error_field {
    ($name:expr, $field:expr) => {
        return Err(syn::Error::new(
            $name.span(),
            format!("`{}` attribute only supports {}", $name, $field),
        ))
    };
}

pub fn apply_environment(
    field_attributes: &mut FieldAttr,
    attribute: &AttributeParse,
) -> syn::Result<()> {
    match attribute.parsed_value {
        super::argument_parser::AttributeTypeParsed::None => {
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Blank => {
            field_attributes.environment = Environment::Auto;
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Single(ref attribute_value) => {
            match attribute_value {
                super::argument_parser::AttributeValue::Int(_) => {
                    return_syn_error_field!(attribute.name_ident, "Blank and Single(Str|Ident)");
                }
                super::argument_parser::AttributeValue::Ident(lit_ident) => {
                    field_attributes.environment = Environment::Ident(lit_ident.clone())
                }
                super::argument_parser::AttributeValue::Str(lit_str) => {
                    field_attributes.environment = Environment::String(lit_str.clone())
                }
            }
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Vector(ref attribute_values) => {
            return_syn_error_field!(attribute.name_ident, "Blank and Single(Str|Ident)");
        }
    }

    return Err(syn::Error::new(
        attribute.name_ident.span(),
        format!("`{}` how did we get here?", attribute.name),
    ));
}

pub fn apply_string(field: &mut FieldAttr, attribute: &AttributeParse) -> syn::Result<()> {
    match attribute.parsed_value {
        super::argument_parser::AttributeTypeParsed::None => {
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Blank => {
            field.string = true;
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Single(_)
        | super::argument_parser::AttributeTypeParsed::Vector(_) => {
            return_syn_error_field!(attribute.name_ident, "Blank");
        }
    }

    return Err(syn::Error::new(
        attribute.name_ident.span(),
        format!("`{}` how did we get here?", attribute.name),
    ));
}

pub fn apply_size_from(field: &mut FieldAttr, attribute: &AttributeParse) -> syn::Result<()> {
    match &attribute.parsed_value {
        super::argument_parser::AttributeTypeParsed::None => {
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Blank => {
            field.set_size = SizeOffset::SizeAuto;
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Single(attribute_value) => {
            match attribute_value {
                super::argument_parser::AttributeValue::Int(lit_int) => {
                    field.set_size = SizeOffset::SizeInt(lit_int.clone());
                    return Ok(());
                }
                super::argument_parser::AttributeValue::Str(lit_str) => {
                    field.set_size = SizeOffset::SizeStr(lit_str.clone());
                    return Ok(());
                }
                super::argument_parser::AttributeValue::Ident(ident) => {
                    return_syn_error_field!(attribute.name_ident, "Blank or Single(Str|Int)");
                }
            };
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Vector(attribute_values) => {
            return_syn_error_field!(attribute.name_ident, "Blank or Single(Str|Int)");
        }
    }

    return Err(syn::Error::new(
        attribute.name_ident.span(),
        format!("`{}` how did we get here?", attribute.name),
    ));
}

pub fn apply_offset_from(field: &mut FieldAttr, attribute: &AttributeParse) -> syn::Result<()> {
    match &attribute.parsed_value {
        super::argument_parser::AttributeTypeParsed::None => {
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Blank => {
            field.set_offset = SizeOffset::OffsetAuto;
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Single(attribute_value) => {
            match attribute_value {
                super::argument_parser::AttributeValue::Int(lit_int) => {
                    field.set_offset = SizeOffset::OffsetInt(lit_int.clone());
                }
                super::argument_parser::AttributeValue::Str(lit_str) => {
                    field.set_offset = SizeOffset::OffsetStr(lit_str.clone());
                }
                super::argument_parser::AttributeValue::Ident(ident) => {
                    return_syn_error_field!(attribute.name_ident, "Blank or Single(Str|Int)");
                }
            };
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Vector(attribute_values) => {
            return_syn_error_field!(attribute.name_ident, "Blank or Single(Str|Int)");
        }
    }
    return Err(syn::Error::new(
        attribute.name_ident.span(),
        format!("`{}` how did we get here?", attribute.name),
    ));
}

pub fn apply_size_offset_from(
    field: &mut FieldAttr,
    attribute: &AttributeParse,
) -> syn::Result<()> {
    match &attribute.parsed_value {
        super::argument_parser::AttributeTypeParsed::None => return Ok(()),
        super::argument_parser::AttributeTypeParsed::Blank => {
            field.set_size = SizeOffset::SizeAuto;
            field.set_offset = SizeOffset::OffsetAuto;
            return Ok(());
        }
        super::argument_parser::AttributeTypeParsed::Single(attribute_value) => {
            match &attribute_value {
                super::argument_parser::AttributeValue::Int(_)
                | super::argument_parser::AttributeValue::Ident(_) => {
                    return_syn_error_field!(
                        attribute.name_ident,
                        "Blank, Single(Str), or Vector(Int, Str)"
                    );
                }
                super::argument_parser::AttributeValue::Str(lit_str) => {
                    field.set_size = SizeOffset::SizeStr(lit_str.clone());
                    field.set_offset = SizeOffset::OffsetStr(lit_str.clone());
                    return Ok(());
                }
            };
        }
        super::argument_parser::AttributeTypeParsed::Vector(attribute_values) => {
            if attribute_values.len() != 2 {
                return_syn_error_field!(
                    attribute.name_ident,
                    format!(
                        "Vector(Int, Str) of a length of 2, got {}",
                        attribute_values.len()
                    )
                );
            }
            let f1 = attribute_values[0].clone();
            let f2 = attribute_values[1].clone();
            match f1 {
                super::argument_parser::AttributeValue::Int(lit_int) => {
                    field.set_size = SizeOffset::SizeInt(lit_int.clone());
                    match f2 {
                        super::argument_parser::AttributeValue::Int(lit_int) => {
                            field.set_offset = SizeOffset::SizeInt(lit_int.clone());
                        }
                        super::argument_parser::AttributeValue::Str(lit_str) => todo!(),
                        super::argument_parser::AttributeValue::Ident(_) => {
                            return_syn_error_field!(
                                attribute.name_ident,
                                "Blank, Single(Str), or Vector(Int, Str)"
                            );
                        }
                    };
                    return Ok(());
                }
                super::argument_parser::AttributeValue::Str(lit_str) => {
                    field.set_size = SizeOffset::SizeStr(lit_str.clone());
                    match f2 {
                        super::argument_parser::AttributeValue::Int(lit_int) => {
                            field.set_offset = SizeOffset::OffsetInt(lit_int.clone());
                        }
                        super::argument_parser::AttributeValue::Str(lit_str) => {
                            field.set_offset = SizeOffset::OffsetStr(lit_str.clone());
                        }
                        super::argument_parser::AttributeValue::Ident(ident) => {
                            return_syn_error_field!(
                                attribute.name_ident,
                                "Blank, Single(Str), or Vector(Int, Str)"
                            );
                        }
                    };
                    return Ok(());
                }
                super::argument_parser::AttributeValue::Ident(ident) => {
                    return_syn_error_field!(
                        attribute.name_ident,
                        "Blank, Single(Str), or Vector(Int, Str)"
                    );
                }
            }
        }
    }

    return Err(syn::Error::new(
        attribute.name_ident.span(),
        format!("`{}` how did we get here?", attribute.name),
    ));
}

pub fn apply_size(field_attributes: &FieldAttr, attribute: &AttributeParse) -> syn::Result<()> {
    Ok(())
}
