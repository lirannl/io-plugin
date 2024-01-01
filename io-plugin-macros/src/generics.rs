use itertools::Itertools;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Variant, ItemEnum, punctuated::Punctuated, Ident};

pub fn variant_type_names(variant: &Variant) -> HashSet<String> {
    variant
        .fields
        .iter()
        .map(|f| f.ty.to_token_stream().to_string())
        .collect()
}

pub fn enum_generics(target: impl Iterator<Item = &Variant>, source: &ItemEnum) -> Punctuated<Ident, syn::token::Comma> {
    let types = target
        .map(variant_type_names)
        .flatten()
        .collect_vec();
    let types = source
        .generics
        .type_params()
        .filter_map(|t| {
            if types.contains(&t.ident.to_string()) {
                Some(t.ident.to_owned())
            } else {
                None
            }
        })
        .collect();
    types
}