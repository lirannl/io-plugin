use std::{collections::HashMap, fmt::Display};

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use regex::Regex;
use syn::{
    braced, parse_quote, parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{self, Brace, Comma},
    Attribute, Block, Ident, Item, ItemEnum, ItemTrait, Stmt, TraitItem,
};

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

pub fn generate_gate(gate: Option<impl Display>) -> Attribute {
    let gate = gate.map(|g| g.to_string());
    parse_quote!(#[cfg(target_feature = #gate)])
}

pub fn generate_trait(
    name: &Ident,
    message: ItemEnum,
    response: ItemEnum,
    gates: HashMap<String, String>,
) -> (ItemTrait, ItemTrait) {
    let client_gate = generate_gate(gates.get("client"));
    let host_gate = generate_gate(gates.get("host"));
    let vis = &message.vis;
    let name = format_ident!("{name}Interface");
    let methods = message.variants.iter().zip(&response.variants);
    let methods = methods
        .map(|(message, response)| -> TraitItem {
            let name = format_ident!("{}", pascal_to_snake(message.ident.to_string()));
            let params = &message.fields;
            if response.fields.len() == 1 {
                let return_type = response.fields.iter().last().map(|f| f.ty.to_owned());
                parse_quote_spanned! {message.span()=>
                fn #name(&self) -> Box<dyn futures::Future<Output = Result<#return_type, Box<dyn std::error::Error>>>>;}
            } else {
                let return_type = &response
                    .fields
                    .iter()
                    .map(|f| f.ty.to_owned())
                    .collect::<Punctuated<_, Comma>>();
                parse_quote_spanned!(message.span()=>
                fn #name(&self) -> Box<dyn futures::Future<Output = Result<(#return_type), Box<dyn std::error::Error>>>>;)
            }
        })
        .collect::<Vec<_>>();
    let mut generated_client: ItemTrait = parse_quote_spanned!(message.span()=>
        #vis trait #name {
        }
    );
    generated_client.attrs.extend_one(client_gate);
    let mut generated_host: ItemTrait = parse_quote_spanned!(message.span()=>
    #vis trait #name {
        #(#methods)*
        }
    );
    // generated_host.attrs.extend_one(host_gate);
    (generated_client, generated_host)
}
