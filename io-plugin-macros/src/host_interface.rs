use std::{collections::HashMap, fmt::Display};

use itertools::izip;
use lazy_static::lazy_static;
use quote::format_ident;
use regex::Regex;
use syn::{
    parse_quote_spanned, punctuated::Punctuated, spanned::Spanned, token::Comma, FnArg, ItemEnum,
    ItemTrait, Meta, MetaList, TraitItem, Variant,
};

use crate::generate_gate;

lazy_static! {
    pub static ref PASCAL_PARTS: Regex = Regex::new("[A-Z0-9_][a-z0-9_]+").unwrap();
}

pub fn pascal_to_snake(pascal: impl Display) -> String {
    let binding = pascal.to_string();
    let parts = PASCAL_PARTS.find_iter(&binding);
    parts
        .map(|p| {
            let mut p = p.as_str().chars();
            let head = p.next().unwrap_or_default().to_lowercase();
            let rest = p.as_str();
            format!("{head}{rest}")
        })
        .collect::<Vec<_>>()
        .join("_")
}

pub fn generate_trait(
    original: ItemEnum,
    message: ItemEnum,
    response: ItemEnum,
    gates: HashMap<String, String>,
) -> ItemTrait {
    let _client_gate = generate_gate(gates.get("client"));
    let host_gate = generate_gate(gates.get("host"));
    let vis = &message.vis;
    let name = format_ident!("{}Interface", original.ident);
    let methods = izip![&original.variants, &message.variants, &response.variants];
    let methods = methods
        .map(|(original, message, response)| -> TraitItem {
            let params = generate_trait_fn_params(original, message);
            generate_trait_fn(original, response, params)
        })
        .collect::<Vec<_>>();
    let mut generated_host: ItemTrait = parse_quote_spanned!(message.span()=>
    #vis trait #name {
        #(#methods)*
        }
    );
    if let Some(host_gate) = host_gate {
        generated_host.attrs.extend_one(host_gate);
    }
    generated_host
}

fn generate_trait_fn(
    original: &Variant,
    response: &Variant,
    params: Punctuated<FnArg, Comma>,
) -> TraitItem {
    let name = format_ident!("{}", pascal_to_snake(original.ident.to_string()));

    if response.fields.len() == 1 {
        let return_type = response.fields.iter().last().map(|f| f.ty.to_owned());
        parse_quote_spanned! {original.span()=>
        fn #name(#params) -> Box<dyn futures::Future<Output = Result<#return_type, Box<dyn std::error::Error>>>>;}
    } else {
        let return_type = &response
            .fields
            .iter()
            .map(|f| f.ty.to_owned())
            .collect::<Punctuated<_, Comma>>();
        parse_quote_spanned!(original.span()=>
        fn #name(#params) -> Box<dyn futures::Future<Output = Result<(#return_type), Box<dyn std::error::Error>>>>;)
    }
}

fn generate_trait_fn_params(original: &Variant, message: &Variant) -> Punctuated<FnArg, Comma> {
    let self_attr = original.attrs.iter().find_map(|a| {
        if let Meta::List(MetaList { path, tokens, .. }) = &a.meta
            && let Some(ident) = path.get_ident()
            && ident.to_string() == "host_trait_self"
        {
            Some(tokens)
        } else {
            None
        }
    });
    eprintln!("{:#?}", self_attr);

    izip![&original.fields, &message.fields]
        .enumerate()
        .map(|(i, (original, message))| -> FnArg {
            let ty = &message.ty;
            let param = format_ident!("arg{}", (i + 1).to_string());
            parse_quote_spanned!(original.span()=>#param: #ty)
        })
        .collect::<Punctuated<_, Comma>>()
}
