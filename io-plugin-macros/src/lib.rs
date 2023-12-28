#![feature(extend_one, let_chains, anonymous_lifetime_in_impl_trait)]
use std::{collections::HashMap, fmt::Display};

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{format_ident, quote_spanned};
use regex::Regex;
use syn::{parse_macro_input, parse_quote, spanned::Spanned, Attribute, ItemEnum};
mod host_interface;
mod variants;

lazy_static! {
    static ref GATES_PARSER: Regex = Regex::new("(\\w+)_gate\\W*=\\W*\"(\\w+)\"").unwrap();
}

fn generate_gate(gate: Option<impl Display>) -> Option<Attribute> {
    let gate = gate?.to_string();
    Some(parse_quote!(#[cfg(target_feature = #gate)]))
}

#[proc_macro_attribute]
pub fn io_plugin(attribute_data: TokenStream, input: TokenStream) -> TokenStream {
    let gates = GATES_PARSER
        .captures_iter(&attribute_data.to_string())
        .filter_map(|gate| {
            Some((
                gate.get(1)?.as_str().to_owned(),
                gate.get(2)?.as_str().to_owned(),
            ))
        })
        .collect::<HashMap<_, _>>();
    let mut input = parse_macro_input!(input as ItemEnum);

    input.ident = format_ident!("{}", input.ident.to_string().trim_start_matches("_"));

    let (message, response) = variants::split_enum(&input);
    let host_interface = host_interface::generate_trait(
        input.clone(),
        message.clone(),
        response.clone(),
        gates.clone(),
    );
    // let _ = write!(
    //     fs::OpenOptions::new()
    //         .append(true)
    //         .create(true)
    //         .open(PathBuf::from_str("/tmp/io_plugin_trait.txt").unwrap()).unwrap(),
    //     "{gates:#?}"
    // );

    quote_spanned!(message.span()=>
    #message
    #response
    #host_interface
    )
    .into()
}
