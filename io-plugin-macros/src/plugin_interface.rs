use itertools::{izip, Itertools};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{
    parse_quote, parse_quote_spanned, punctuated::Punctuated, spanned::Spanned, token::Comma, Arm,
    Expr, ItemEnum, ItemFn, ItemTrait, Pat, TraitItemFn, Type, Path, Ident,
};

use crate::{
    handle::pascal_to_snake,
    util::{get_doc, list_attr_by_id},
};

pub fn generate_trait(
    original: ItemEnum,
    message: ItemEnum,
    response: ItemEnum,
    // _gates: HashMap<String, String>,
) -> (ItemTrait, ItemFn) {
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
        .map(|(original_v, message_v, response_v)| -> TraitItemFn {
            let name = format_ident!("{}", pascal_to_snake(original_v.ident.to_string()));

            let args = message_v
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| -> (Ident, Type) {
                    let name = format_ident!("arg{}", i + 1);
                    let ty = &f.ty;
                    (parse_quote_spanned! {f.span()=>#name}, ty.to_owned())
                })
                .collect_vec();
            let arg_idents = args.iter().map(|(id, _)| id).collect_vec();
            let fn_args = args.iter().map(|(id, ty)| quote!(#id: #ty));

            let return_type: Type = {
                let types = response_v
                    .fields
                    .iter()
                    .map(|f| f.ty.to_owned())
                    .collect::<Punctuated<_, Comma>>();
                if let Some(ty) = types.first()
                    && types.len() == 1
                {
                    ty.to_owned()
                } else {
                    parse_quote_spanned!(original_v.span()=>(#types))
                }
            };
            let doc = get_doc(original_v);
            let implementation = 
            if let Some((_, content)) = list_attr_by_id(original_v.attrs.as_slice(), "implementation") {
                let implementation: Path = parse_quote!(#content);

                if {
                    let types = args.iter().map(|t| t.1.to_token_stream().to_string()).collect_vec();
                    original.generics.type_params().any(|g| types.contains(&g.ident.to_string()))
                }
                {quote_spanned!(original_v.fields.span()=>{compile_error!("generics are not supported in plugin methods with default implementations")})}
                else 
{
                quote_spanned!(implementation.span()=>{ async move {
                    #implementation(self, #(#arg_idents),*).await
                }})
            }
            } else {
                quote!(;)
            }
            ;
            parse_quote_spanned!(original_v.span()=>
            #doc
            fn #name(&mut self, #(#fn_args),*) -> impl std::future::Future<Output = Result<#return_type, Box<dyn std::error::Error>>> where Self: Sized #implementation)
        })
        .collect::<Vec<_>>();

    let message_generics = &message
        .generics
        .type_params()
        .map(|t| t.ident.to_owned())
        .collect_vec();

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
                    parse_quote!(#ty::<#(#message_generics),*>::#v(#message_idents))
                } else {
                    parse_quote!(#ty::<#(#message_generics),*>::#v)
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
                match plugin.#method_ident(#message_idents).await {
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
    let mut generics = original.generics.clone();
    for ty in generics.type_params_mut() {
        ty.default = None;
    }
    let generics = generics.type_params().collect_vec();
    let generic_idents = generics.iter().map(|p| p.ident.to_owned());

    (
        parse_quote_spanned!(original.span()=>
        #[doc=#plugin_trait_doc]
        #vis trait #name <#(#generics),*> {
            #(#methods)*
            ///Generally, you'd want to call this in the "main" func - as this starts the plugin
            fn main_loop(mut self) -> impl std::future::Future<Output = ()> where Self: Sized { async move {
                    let mut stdin = io_plugin::stdin();
                    let mut stdout = io_plugin::stdout();

                    loop {
                        __main_loop_iteration(&mut self, &mut stdin, &mut stdout).await
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
                }}
        }),
        parse_quote!(///Only used internally to iterate through recieving and then responding to messages. Call [`self::main_loop`] instead
        async fn __main_loop_iteration <#(#generics),*, ___Plugin___: #name<#(#generic_idents),*>> (plugin: &mut ___Plugin___, stdin: &mut io_plugin::Stdin, stdout: &mut io_plugin::Stdout) -> Result<(), Box<dyn std::error::Error>> {
                let message: #message_name <#(#message_generics),*> = io_plugin::io_read_async(std::pin::pin!(stdin.get_mut())).await?;
                let response = match message {
                    #(#arms)*
                };
                io_plugin::io_write_async(std::pin::pin!(stdout), response).await?;
                Ok(())
            }),
    )
}
