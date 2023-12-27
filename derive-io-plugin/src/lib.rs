#![feature(proc_macro_quote, extend_one, let_chains)]

use proc_macro::{TokenStream, quote};
use quote::format_ident;
use syn::{DeriveInput, Data, parse_macro_input};

// use crate::variants::split_enum_variants;
mod variants;

#[proc_macro_derive(IoPlugin)]
pub fn io_plugin_derive(input: TokenStream) -> TokenStream { 
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    
    let name = &input.ident;
    let message_name = format_ident!("{name}Message");
    let response_name = format_ident!("{name}Response");
    let data = match &input.data {
        Data::Enum(data) => data,
        _ => panic!("IoPlugin can only be derived for enums")
    };
    
    // let (message_variants, response_variants) = split_enum_variants(data.variants);   
    

    quote!(
        pub enum #message_name {
            #message_variants
        }
        pub enum #response_name {
            #response_variants
        }
    )
}