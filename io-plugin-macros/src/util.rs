use std::fmt::Display;

use lazy_static::lazy_static;
use quote::quote;
use regex::Regex;
use syn::{parse_quote, Attribute, Expr, Ident, Meta, MetaList, MetaNameValue, Variant};

pub fn list_attr_by_id(
    original: &[Attribute],
    id: &str,
) -> Option<(Ident, proc_macro2::TokenStream)> {
    original.iter().find_map(|a| {
        if let Meta::List(MetaList { path, tokens, .. }) = &a.meta
            && let Some(ident) = path.get_ident()
            && ident.to_string() == id
        {
            Some((ident.to_owned(), tokens.to_owned()))
        } else {
            None
        }
    })
}

pub fn name_value_attr_by_name(original: &[Attribute], id: &str) -> Option<(Ident, Expr)> {
    original.iter().find_map(|a| {
        if let Meta::NameValue(MetaNameValue { path, value, .. }) = &a.meta
            && let Some(ident) = path.get_ident()
            && ident.to_string() == id
        {
            Some((ident.to_owned(), value.to_owned()))
        } else {
            None
        }
    })
}

pub fn get_doc(variant: &Variant) -> Option<proc_macro2::TokenStream> {
    let doc = name_value_attr_by_name(variant.attrs.as_slice(), "doc");
    if let Some((ident, value)) = doc {
        Some(quote!(#[#ident = #value]))
    } else {
        None
    }
}

lazy_static! {
    pub static ref GATES_PARSER: Regex = Regex::new("(\\w+)_gate\\W*=\\W*\"(\\w+)\"").unwrap();
}

pub fn generate_gate(gate: Option<impl Display>) -> Option<Attribute> {
    let gate = gate?.to_string();
    Some(parse_quote!(#[cfg(target_feature = #gate)]))
}
