#![feature(extend_one, let_chains, anonymous_lifetime_in_impl_trait, extract_if)]

use proc_macro::TokenStream;
use quote::{format_ident, quote_spanned};
use std::collections::HashMap;
use syn::{parse_macro_input, spanned::Spanned, ItemEnum};
use util::GATES_PARSER;

mod enums;
mod generics;
mod handle;
mod plugin_interface;
mod util;

/// Generate a plugin-interface, based on an enum definition for its' operations
/// From the plugin's perspective - input types are all fields except the last one, and the output type is the last one 
/// (of course - you can use tuples to output multiple values).
///
/// The provided enum's variant must contain only owned data (no &'a) - otherwise, deserialiastaion will cause a compile-time error.
/// The variants must be [`serde::Serialize`] + [`serde::Deserialize`].
/// 
/// Note that the enum this attribute applies to won't exist.  
/// Instead, there will be a `message` enum, `response` enum, plugin `trait`, plugin `handle` (a struct) - postfixed with the highlighted words.
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
    if let Some(lifetime) = input.generics.lifetimes().last() {
        return quote_spanned!(lifetime.span()=>compile_error!("lifetimes are not supported in `io_plugin`");).into();
    }

    input.ident = format_ident!("{}", input.ident.to_string().trim_start_matches("_"));

    let (message, response, response_impl) = enums::split_enum(&mut input);

    for ty in input.generics.type_params_mut() {
        ty.default = None;
    }

    #[allow(unused_variables)]
    let handle = handle::generate_handle(
        input.clone(),
        message.clone(),
        response.clone(),
        gates.clone(),
    );

    let (plugin_interface, main_loop_iteration) = plugin_interface::generate_trait(
        input.clone(),
        message.clone(),
        response.clone(),
        gates.clone(),
    );

    quote_spanned!(message.span()=>
    #message

    #response
    #response_impl

    #plugin_interface
    #main_loop_iteration

    #handle
    )
    .into()
}

/// Allows customising the documentation of the handle generated by [`io_plugin`]
#[proc_macro_attribute]
pub fn handle_doc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Allows customising the documentation of the plugin trait generated by [`io_plugin`]
#[proc_macro_attribute]
pub fn plugin_trait_doc(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Attributes which only apply to the plugin message enum
#[proc_macro_attribute]
pub fn message_attributes(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Attributes which only apply to the plugin response enum
#[proc_macro_attribute]
pub fn response_attributes(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
