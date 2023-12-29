use quote::{format_ident, quote};
use syn::{
    parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Semi},
    Arm, ItemEnum, ItemImpl, Type, Variant,
};

use crate::util::get_doc;

type EnumVariants = Punctuated<Variant, Comma>;

pub fn split_enum(input: &ItemEnum) -> (ItemEnum, ItemEnum, ItemImpl) {
    let vis = &input.vis;
    let attrs = input.attrs.iter().collect::<Punctuated<_, Semi>>();
    let (mut message_variants, mut response_variants) = (EnumVariants::new(), EnumVariants::new());
    for variant in input.variants.iter() {
        let name = &variant.ident;

        let doc = get_doc(variant);

        let mut fields = variant.fields.iter().collect::<Vec<_>>();

        let response: Variant = if let Some(field) = fields.pop() {
            match &field.ty {
                Type::Tuple(types) if types.elems.len() > 0 => {
                    let types = &types.elems;
                    parse_quote_spanned!(
                        variant.span()=>
                        #doc
                        #name(#types))
                }
                Type::Tuple(_) => parse_quote_spanned!(variant.span()=>#name),
                _ => {
                    let ty = &field.ty;
                    parse_quote_spanned!(
                        variant.span()=>
                        #doc
                        #name(#ty))
                }
            }
        } else {
            parse_quote_spanned!(variant.span()=>#name)
        };
        let message_types = fields
            .iter()
            .map(|f| f.ty.to_owned())
            .collect::<Punctuated<_, Comma>>();

        let new_variant: Variant = if message_types.len() == 0 {
            parse_quote_spanned!(variant.span()=>
            #doc
            #name)
        } else {
            parse_quote_spanned!(variant.span()=>
            #doc
            #name (#message_types))
        };
        message_variants.extend_one(new_variant);
        response_variants.extend_one(response);
    }

    let message_name = format_ident!("{}Message", &input.ident);
    let response_name = format_ident!("{}Response", &input.ident);

    let response_variant_arms = response_variants
        .iter()
        .map(|variant| -> Arm {
            let name = &variant.ident;
            let name_str = name.to_string();
            let fields = variant
                .fields
                .iter()
                .map(|_| quote!(_))
                .collect::<Punctuated<_, Comma>>();
            if fields.len() > 0 {
                parse_quote_spanned!(variant.span()=>#response_name::#name(#fields) => #name_str,)
            } else {
                parse_quote_spanned!(variant.span()=>#response_name::#name => #name_str,)
            }
        })
        .collect::<Vec<_>>();

    (
        parse_quote_spanned!(input.span()=>
            #[forbid(non_camel_case_types)]
            #[derive(serde::Deserialize, serde::Serialize)]
            #attrs
            #vis enum #message_name {
                #message_variants
            }
        ),
        parse_quote_spanned!(input.span()=>
            #[forbid(non_camel_case_types)]
            #[derive(serde::Deserialize, serde::Serialize)]
            #attrs
            #vis enum #response_name {
                #response_variants
            }
        ),
        parse_quote_spanned!(input.span()=>impl #response_name {
            #[allow(dead_code)]
            #vis fn variant_name(&self) -> &'static str {
                match self {
                    #(#response_variant_arms)*
                }
            }
        }),
    )
}
