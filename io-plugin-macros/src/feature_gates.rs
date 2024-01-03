use quote::ToTokens;
use std::collections::HashMap;
use syn::{parse::Parse, punctuated::Punctuated, token::Comma, ExprAssign};

pub struct FeatureGates(Punctuated<ExprAssign, Comma>);
impl Parse for FeatureGates {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(Punctuated::<ExprAssign, Comma>::parse_terminated(
            input,
        )?))
    }
}
impl ToTokens for FeatureGates {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.0.to_tokens(tokens)
    }
}
impl FeatureGates {
    pub fn hashmap(&self) -> HashMap<String, String> {
        self.0
            .iter()
            .map(|kv| {
                (
                    kv.left.to_token_stream().to_string(),
                    kv.right.to_token_stream().to_string(),
                )
            })
            .collect()
    }
}
