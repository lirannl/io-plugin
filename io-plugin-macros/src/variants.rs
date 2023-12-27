use quote::format_ident;
use syn::{
    parse_quote_spanned,
    punctuated::Punctuated,
    spanned::Spanned,
    token::{Comma, Semi},
    ItemEnum, Type, Variant,
};

type EnumVariants = Punctuated<Variant, Comma>;

pub fn split_enum(input: &ItemEnum) -> (ItemEnum, ItemEnum) {
    let vis = &input.vis;
    let attrs = input.attrs.iter().collect::<Punctuated<_, Semi>>();
    let (mut message_variants, mut response_variants) = (EnumVariants::new(), EnumVariants::new());
    for variant in input.variants.iter() {
        let name = &variant.ident;
        let mut fields = variant.fields.iter().collect::<Vec<_>>();

        let response: Variant = if let Some(field) = fields.pop() {
            match &field.ty {
                Type::Tuple(types) if types.elems.len() > 0 => {
                    let types = &types.elems;
                    parse_quote_spanned!(variant.span()=>#name(#types))
                }
                Type::Tuple(_) => parse_quote_spanned!(variant.span()=>#name),
                _ => {
                    let ty = &field.ty;
                    parse_quote_spanned!(variant.span()=>#name(#ty))
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
            parse_quote_spanned!(variant.span()=>#name)
        } else {
            parse_quote_spanned!(variant.span()=>#name (#message_types))
        };
        message_variants.extend_one(new_variant);
        response_variants.extend_one(response);
    }

    let message_name = format_ident!("{}Message", &input.ident);
    let response_name = format_ident!("{}Response", &input.ident);
    (
        parse_quote_spanned!(input.span()=>
            #[derive(serde::Deserialize, serde::Serialize)]
            #attrs
            #vis enum #message_name {
                #message_variants
            }
        ),
        parse_quote_spanned!(input.span()=>
            #[derive(serde::Deserialize, serde::Serialize)]
            #attrs
            #vis enum #response_name {
                #response_variants
            }
        ),
    )
}
