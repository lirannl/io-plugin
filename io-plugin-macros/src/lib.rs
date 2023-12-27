#![feature(extend_one, let_chains)]
use std::collections::HashMap;

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{format_ident, quote_spanned};
use regex::Regex;
use syn::{parse_macro_input, spanned::Spanned, ItemEnum};
mod interface;
mod variants;

lazy_static! {
    static ref GATES_PARSER: Regex = Regex::new("(\\w+)_gate\\W*=\\W*\"(\\w+)\"").unwrap();
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
    let (client_interface, host_interface) =
        interface::generate_trait(&input.ident, message.clone(), response.clone(), gates);

    quote_spanned!(message.span()=>
    #message
    #response
    #client_interface
    #host_interface
    )
    .into()
}
