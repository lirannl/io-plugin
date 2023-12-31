use std::collections::HashMap;

use itertools::izip;
use quote::format_ident;
use syn::{
    parse_quote, parse_quote_spanned, punctuated::Punctuated, spanned::Spanned, token::Comma, Arm,
    Expr, FnArg, ItemEnum, ItemTrait, Pat, TraitItemFn, Type,
};

use crate::{
    handle::pascal_to_snake,
    util::{get_doc, list_attr_by_id},
};

pub fn generate_trait(
    original: ItemEnum,
    message: ItemEnum,
    response: ItemEnum,
    _gates: HashMap<String, String>,
) -> ItemTrait {
    let name = format_ident!("{}Trait", original.ident);
    let vis = &original.vis;
    let variants = izip![
        original.variants.to_owned(),
        message.variants.to_owned(),
        response.variants.to_owned()
    ]
    .collect::<Vec<_>>();
    let methods = variants
        .iter()
        .map(|(original, message, response)| -> TraitItemFn {
            let name = format_ident!("{}", pascal_to_snake(original.ident.to_string()));

            let args = message
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| -> FnArg {
                    let name = format_ident!("arg{}", i + 1);
                    let ty = &f.ty;
                    parse_quote_spanned! {f.span()=>#name: #ty}
                })
                .collect::<Punctuated<_, Comma>>();

            let return_type: Type = {
                let types = response
                    .fields
                    .iter()
                    .map(|f| f.ty.to_owned())
                    .collect::<Punctuated<_, Comma>>();
                if let Some(ty) = types.first()
                    && types.len() == 1
                {
                    ty.to_owned()
                } else {
                    parse_quote_spanned!(original.span()=>(#types))
                }
            };
            let doc = get_doc(original);

            parse_quote_spanned!(original.span()=>
            #doc
            fn #name(&mut self, #args) -> Result<#return_type, Box<dyn std::error::Error>>;)
        })
        .collect::<Vec<_>>();

    let arms = variants
        .iter()
        .zip(&methods)
        .map(|((original_v, message_v, response_v), method)| -> Arm {
            let message_idents = message_v
                .fields
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("arg{}", i + 1))
                .collect::<Punctuated<_, Comma>>();

            let response_idents = response_v
                .fields
                .iter()
                .enumerate()
                .map(|(i, _)| format_ident!("arg{}", i + 1))
                .collect::<Punctuated<_, Comma>>();

            let pat: Pat = {
                let ty = &message.ident;
                let v = &message_v.ident;
                if message_idents.len() > 0 {
                    parse_quote!(#ty::#v(#message_idents))
                } else {
                    parse_quote!(#ty::#v)
                }
            };
            let return_expr: Expr = {
                let ty = &response.ident;
                let v = &response_v.ident;
                if response_idents.len() > 0 {
                    parse_quote!(#ty::#v(#response_idents))
                } else {
                    parse_quote!(#ty::#v)
                }
            };
            let method_ident = &method.sig.ident;
            let arm = parse_quote_spanned!(original_v.span()=>
            #pat => {
                match self.#method_ident(#message_idents) {
                    #[allow(unused_parens)]
                    Ok((#response_idents)) => Ok(#return_expr),
                    Err(err) => Err(io_plugin::IOPluginError::Other(err.to_string())),
                }
            });
            arm
        })
        .collect::<Vec<_>>();

    let message_name = &message.ident;

    let plugin_trait_doc = if let Some((_, doc)) =
        list_attr_by_id(&original.attrs, "plugin_trait_doc")
    {
        let doc = doc.to_string();
        doc[1..doc.len() - 1].to_owned()
    } else {
        format!("This trait defines the plugin executable's interface. To use, implement it on a struct, and call [`{name}::main_loop`] (generally in the main function)")
    };
    parse_quote_spanned!(original.span()=>
    #[doc=#plugin_trait_doc]
    #vis trait #name {
        #(#methods)*
        fn main_loop(mut self) -> ! where Self: Sized {
                    let mut stdin = std::io::BufReader::new(std::io::stdin());
                    let mut stdout = std::io::stdout();

                    loop {
                        (|| -> Result<(), Box<dyn std::error::Error>> {
                            let message: #message_name = io_plugin::io_read(stdin.get_mut())?;
                            let response = match message {
                                #(#arms)*
                            };
                            io_plugin::io_write(&mut stdout, response)?;
                            Ok(())
                        })()
                        .unwrap_or_else(|err| {
                            if let Some(&io_plugin::IOPluginError::PipeClosed) =
                                err.downcast_ref::<io_plugin::IOPluginError>()
                            {
                                eprintln!("Host closed");
                                std::process::exit(0);
                            }
                            eprintln!("{err:#?}")
                        });
                    }
                }
            }
        )
}
