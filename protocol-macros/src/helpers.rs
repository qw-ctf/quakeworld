use std::collections::HashMap;

use syn::Attribute;

pub fn check_tag_value_new(
    attrs: &[Attribute],
    tag: &str,
) -> (bool, HashMap<String, Vec<syn::Lit>>) {
    let mut s: HashMap<String, Vec<syn::Lit>> = HashMap::new();
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
            if let syn::Meta::List(list) = meta {
                if list.path.is_ident(tag) {
                    // The specified attribute is present, extract its value
                    for nested_meta in list.nested {
                        if let syn::NestedMeta::Meta(syn::Meta::List(something)) =
                            nested_meta.clone()
                        {
                            if let Some(ident) = &something.path.get_ident() {
                                let mut types: Vec<syn::Lit> = Vec::new();
                                types = something
                                    .nested
                                    .into_iter()
                                    .filter_map(|l| {
                                        if let syn::NestedMeta::Lit(s) = l {
                                            Some(s)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                s.insert(ident.to_string(), types);
                            }
                        }
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = nested_meta
                        {
                            if let Some(name) = &name_value.path.get_ident() {
                                s.insert(name.to_string(), vec![name_value.lit]);
                            }
                            // if let syn::Lit::Paren(paren) = &name_value.lit {
                            //     for nested_lit in &paren.elems {
                            //         if let syn::Lit::Type(ty) = nested_lit {
                            //             other_types.push(ty.clone());
                            //         }
                            //     }
                            // }
                        }
                        return (true, s);
                    }
                }
            }
        }
    }
    (false, s)
}

pub fn check_tag_value(attrs: &[Attribute], tag: &str) -> (bool, HashMap<String, syn::Lit>) {
    let mut s: HashMap<String, syn::Lit> = HashMap::new();
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
            if let syn::Meta::List(list) = meta {
                if list.path.is_ident(tag) {
                    // The specified attribute is present, extract its value
                    for nested_meta in list.nested {
                        if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = nested_meta
                        {
                            if let Some(name) = name_value.path.get_ident() {
                                s.insert(name.to_string(), name_value.lit);
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

pub fn check_tag(attrs: &[Attribute], tag: &str) -> bool {
    for attr in attrs {
        if let Ok(meta) = attr.parse_meta() {
            if let syn::Meta::Path(name_value) = meta {
                if name_value.is_ident(tag) {
                    return true;
                }
            }
        }
    }
    false
}
