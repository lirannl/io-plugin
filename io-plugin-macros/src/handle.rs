use itertools::{izip, Itertools};
use lazy_static::lazy_static;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use regex::Regex;
use std::{collections::HashSet, fmt::Display};
use syn::{
    parse_quote, parse_quote_spanned, punctuated::Punctuated, spanned::Spanned, token::Comma,
    FnArg, Generics, Ident, ImplItemFn, ItemEnum, ItemStruct, Type, Variant, Attribute,
};

use crate::util::{get_doc, list_attr_by_id};

lazy_static! {
    pub static ref PASCAL_PARTS: Regex = Regex::new("[A-Z0-9_][a-z0-9_]+").unwrap();
}

pub fn pascal_to_snake(pascal: impl Display) -> String {
    let binding = pascal.to_string();
    let parts = PASCAL_PARTS.find_iter(&binding).collect::<Vec<_>>();
    parts
        .iter()
        .map(|p| {
            let mut p = p.as_str().chars();
            let head = p.next().unwrap_or_default().to_lowercase();
            let rest = p.as_str();
            format!("{head}{rest}")
        })
        .collect::<Vec<_>>()
        .join("_")
}

pub fn generate_handle(
    original: ItemEnum,
    message: ItemEnum,
    response: ItemEnum,
    gate: Option<Attribute>,
) -> TokenStream {
    // let host_gate = generate_gate(gates.get("host"));
    let vis = &message.vis;
    let plugin_name = &original.ident;
    let name = format_ident!("{}Handle", plugin_name);
    let message_ident = &message.ident;
    let response_ident = &response.ident;

    let generics = &original.generics;

    let methods = izip![&original.variants, &message.variants, &response.variants];
    let methods = methods
        .map(|(original, message, response)| {
            let params = generate_method_args(original, message);
            generate_method(
                original,
                message,
                message_ident,
                response,
                response_ident,
                params,
                generics,
            )
        })
        .collect::<Vec<_>>();

    let handle_doc = if let Some((_, doc)) = list_attr_by_id(&original.attrs, "handle_doc") {
        let doc = doc.to_string();
        doc[1..doc.len() - 1].to_owned()
    } else {
        let article = if Regex::new("^[aeiouAEIOU]")
            .unwrap()
            .is_match(&plugin_name.to_string())
        {
            "An"
        } else {
            "A"
        };
        format!("{article} `{plugin_name}` handle on the host")
    };

    let generated_host: ItemStruct = parse_quote_spanned!(message.span()=>
    #[doc = #handle_doc]
    #vis struct #name {
            pub stdio: io_plugin::Mutex<io_plugin::ChildStdio>,
            pub name: std::string::String,
            pub process: io_plugin::Child,
            pub path: std::path::PathBuf,
        }
    );

    let (name_expr, name_param) = if let Some(get_name) = methods
        .iter()
        .find(|m| m.sig.ident.to_string() == "get_name")
        && get_name.sig.inputs.len() == 1
    {
        let get_name = &get_name.sig.ident;
        (quote!(handle.#get_name().await?), None)
    } else {
        (quote!(name), Some(quote!(name: String)))
    };
    let generics = &original.generics.params;
    let message_generics = message
        .generics
        .type_params()
        .map(|g| g.ident.to_owned())
        .collect_vec();
    let response_generics = response
        .generics
        .type_params()
        .map(|g| g.ident.to_owned())
        .collect_vec();
    let handle_impl = quote!(impl #name {
        async fn message<#generics>(&mut self, message: #message_ident <#(#message_generics),*>) -> Result<#response_ident<#(#response_generics),*>, Box<dyn std::error::Error>> {
            let stdio = &self.stdio;
            let mut stdio = stdio.lock().await;
            io_plugin::io_write_async(std::pin::pin!(&mut stdio.stdin), message).await?;
            Ok(
                io_plugin::io_read_async::<Result<_, io_plugin::IOPluginError>>(std::pin::pin!(
                    &mut stdio.stdout
                ))
                .await??,
            )
        }
        pub async fn new(path: std::path::PathBuf, #name_param) -> Result<Self, Box<dyn std::error::Error>> {
            let mut process = io_plugin::spawn_process(&path)?;
            let stdio = process
                .stdin
                .take()
                .and_then(|stdin| {
                    Some(io_plugin::ChildStdio {
                        stdin,
                        stdout: process.stdout.take()?,
                    })
                })
                .ok_or(io_plugin::IOPluginError::InitialisationError(
                    "Stdin/stdout have not been piped".to_string(),
                ))?;
            #[allow(unused_mut)]
            let mut handle = Self {
                process,
                stdio: io_plugin::Mutex::new(stdio),
                name: "".to_string(),
                path
            };
            handle.name = #name_expr;
            Ok(handle)
        }
        #(#methods)*
    });

    quote!(
        #gate
        #generated_host
        
        #gate
        #handle_impl
    )
}

fn generate_method(
    original: &Variant,
    message: &Variant,
    message_type: &Ident,
    response: &Variant,
    response_type: &Ident,
    params: Punctuated<FnArg, Comma>,
    generics: &Generics,
) -> ImplItemFn {
    let name = format_ident!("{}", pascal_to_snake(original.ident.to_string()));
    let message_variant_name = &message.ident;
    let response_variant_name = &response.ident;

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
    let message_fields = if message.fields.len() == 0 {
        None
    } else {
        let fields = message
            .fields
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let name = format_ident!("arg{}", i + 1);
                quote!(#name)
            })
            .collect::<Punctuated<_, Comma>>();
        Some(quote!((#fields)))
    };
    let response_fields = if response.fields.len() == 0 {
        None
    } else {
        let fields = response
            .fields
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let name = format_ident!("arg{}", i + 1);
                quote!(#name)
            })
            .collect::<Punctuated<_, Comma>>();
        Some(quote!((#fields)))
    };
    let ok = match response.fields.iter().collect::<Vec<_>>()[..] {
        [_] => {
            let id = format_ident!("arg{}", "1");
            quote!(Ok(#id))
        }
        [] => {
            quote!(Ok(()))
        }
        _ => quote!(Ok(#response_fields)),
    };

    let doc = get_doc(original);

    let method_generics = {
        let generics = generics.type_params();
        let types = original
            .fields
            .iter()
            .filter_map(|f| Some(f.ty.to_token_stream().to_string()))
            .collect::<HashSet<_>>();
        generics
            .filter(|g| types.contains(&g.ident.to_string()))
            .collect_vec()
    };
    let de_generic = quote!(io_plugin::GenericValue);
    let generics = {
        let mut types = message
            .fields
            .iter()
            .filter_map(|f| Some(f.ty.to_token_stream().to_string()))
            .collect::<HashSet<_>>();
        types.extend(
            response
                .fields
                .iter()
                .filter_map(|f| Some(f.ty.to_token_stream().to_string())),
        );
        generics
            .type_params()
            .map(|tp| {
                if types.contains(&tp.ident.to_string()) {
                    tp.ident.to_token_stream()
                } else {
                    de_generic.clone()
                }
            })
            .collect_vec()
    };

    parse_quote_spanned!(original.ident.span()=>
    #[allow(unreachable_patterns)]
    #doc
    pub async fn #name<#(#method_generics),*>(#params) -> Result<#return_type, Box<dyn std::error::Error>> {
        let response = self.message::<#(#generics),*>(#message_type::#message_variant_name/* */#message_fields).await;
        match response {
            Ok(#response_type::#response_variant_name/* */#response_fields) => #ok,
            Err(e) => Err(e),
            Ok(r) => {
                let res = std::fmt::format(
                    format_args!(
                        "Received {0}. Inappropriate variant",
                        r.variant_name(),
                    ),
                );
                Err(res.into())
            }
        }
    })
}

fn generate_method_args(original: &Variant, message: &Variant) -> Punctuated<FnArg, Comma> {
    let mut args = izip![&original.fields, &message.fields]
        .enumerate()
        .map(|(i, (original, message))| -> FnArg {
            let ty = &message.ty;
            let param = format_ident!("arg{}", (i + 1).to_string());
            parse_quote_spanned!(original.span()=>#param: #ty)
        })
        .collect::<Punctuated<_, Comma>>();

    let arg = parse_quote!(&mut self);
    args.insert(0, arg);
    args
}
