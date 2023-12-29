use std::fmt::Display;

use lazy_static::lazy_static;
use regex::Regex;
use syn::{parse_quote, Attribute};

// pub fn list_attr_by_id(
//     original: &[Attribute],
//     id: &str,
// ) -> Option<(Ident, proc_macro2::TokenStream)> {
//     original.iter().find_map(|a| {
//         if let Meta::List(MetaList { path, tokens, .. }) = &a.meta
//             && let Some(ident) = path.get_ident()
//             && ident.to_string() == id
//         {
//             Some((ident.to_owned(), tokens.to_owned()))
//         } else {
//             None
//         }
//     })
// }

lazy_static! {
    pub static ref GATES_PARSER: Regex = Regex::new("(\\w+)_gate\\W*=\\W*\"(\\w+)\"").unwrap();
}

pub fn generate_gate(gate: Option<impl Display>) -> Option<Attribute> {
    let gate = gate?.to_string();
    Some(parse_quote!(#[cfg(target_feature = #gate)]))
}
