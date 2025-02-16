use std::collections::HashMap;

use proc_macro::Span;
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
        if attr.path.is_ident(tag) {
            if let Ok(meta) = attr.parse_meta() {
                if let syn::Meta::List(list) = meta {
                    // The specified attribute is present, extract its value
                    // println!(
                    //     "\n---> name({}): {:?} -- {:?}\n\t\t\t{:?}\n\n",
                    //     tag,
                    //     list.nested.len(),
                    //     list.path,
                    //     list.nested,
                    // );

                    for nested_meta in list.nested {
                        // println!("----> nested entry:\n\t\t{:?}\n", nested_meta.clone());
                        match nested_meta.clone() {
                            // syn::NestedMeta::Meta(meta) => println!("meta: {:?}", meta),
                            syn::NestedMeta::Meta(meta) => match meta {
                                syn::Meta::Path(path) => {
                                    if let Some(name) = path.get_ident() {
                                        s.insert(
                                            name.to_string(),
                                            syn::Lit::Bool(syn::LitBool::new(
                                                true,
                                                Span::call_site().into(),
                                            )),
                                        );
                                    }
                                }
                                syn::Meta::List(meta_list) => {
                                    // println!("list: {:?}", meta_list.path)
                                }
                                syn::Meta::NameValue(meta_name_value) => {
                                    if let Some(name) = meta_name_value.path.get_ident() {
                                        s.insert(name.to_string(), meta_name_value.lit);
                                        // println!("nv: {:?}", meta_name_value.path)
                                    }
                                }
                            },
                            syn::NestedMeta::Lit(_lit) => {} //println!("lit: {:?}", lit),
                        }
                        // match nested_meta.clone() {
                        //     syn::NestedMeta::Meta(meta) => println!("\tmeta: {:?}", meta),
                        //     syn::NestedMeta::Lit(lit) => println!("\tlit: {:?}", lit),
                        // };
                        // println!("\t---> nested ({}): {:?}", tag, nested_meta);
                        // if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = nested_meta
                        // {
                        //     if let Some(name) = name_value.path.get_ident() {
                        //         s.insert(name.to_string(), name_value.lit);
                        //     }
                        // }
                    }
                    return (true, s);
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

pub fn get_tag_values(attrs: &[Attribute], tag: &str) -> HashMap<String, syn::Lit> {
    let mut s: HashMap<String, syn::Lit> = HashMap::new();
    for attr in attrs {
        if attr.path.is_ident(tag) {
            match attr.parse_meta() {
                Ok(meta) => match meta {
                    syn::Meta::Path(path) => println!("we got a path {:?}", path),
                    syn::Meta::List(meta_list) => {
                        for nested_meta in meta_list.nested {
                            match nested_meta {
                                syn::NestedMeta::Meta(meta) => println!("nested_list: {:?}", meta),
                                syn::NestedMeta::Lit(lit) => println!("literal: {:?}", lit),
                            };
                        }
                        // println!("we got a list {:?}", meta_list);
                    }
                    syn::Meta::NameValue(meta_name_value) => println!("we got a named value"),
                },
                Err(e) => panic!("{}", e),
            }
        }
    }
    s
}
