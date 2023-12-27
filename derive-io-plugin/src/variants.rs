use quote::{quote_spanned, ToTokens};
use syn::{
    parse_quote_spanned, punctuated::Punctuated, spanned::Spanned, token::Comma, Fields,
    FieldsUnnamed, Variant,
};

// pub fn split_enum_variants(variants: Punctuated<Variant, Comma>) -> Vec<(Variant, Variant)> {
//     let vars = variants.iter().map(|variant| {
//         let fields = variant.fields.iter().collect::<Vec<_>>();
//         let response_type = fields
//             .last()
//             .expect(&format!(
//                 "Variant {} must have at least 1 result type",
//                 variant.ident
//             ))
//             .ty;
//         let response_fields: Fields = match response_type {
//             syn::Type::Tuple(t) => {
//                 let types = &t.elems;

//                 if types.is_empty() {
//                     Fields::Unit
//                 } else {
//                     let mut tokens = t.to_token_stream();
//                     Fields::Unnamed(FieldsUnnamed {})
//                 }
//             }
//             _ => Fields::Unnamed(response_type),
//         };
//         (
//             parse_quote_spanned!(variant.span()=>
//                 fields.iter().skip(1)
//             ),
//             parse_quote_spanned!(variant.span()=>

//             ),
//         )
//     });
//     todo!()
// }
