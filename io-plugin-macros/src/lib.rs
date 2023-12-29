#![feature(extend_one, let_chains, anonymous_lifetime_in_impl_trait)]

use proc_macro::TokenStream;
use quote::{format_ident, quote_spanned};
use std::collections::HashMap;
use syn::{parse_macro_input, spanned::Spanned, ItemEnum};
use util::GATES_PARSER;
mod handle_interface;
mod util;
mod variants;
mod plugin_interface;

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

    let (message, response, response_impl) = variants::split_enum(&input);
    let host_interface = handle_interface::generate_trait(
        input.clone(),
        message.clone(),
        response.clone(),
        gates.clone(),
    );
    let plugin_interface = plugin_interface::generate_trait(
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
    #response_impl
    #host_interface
    #plugin_interface
    )
    .into()
}
