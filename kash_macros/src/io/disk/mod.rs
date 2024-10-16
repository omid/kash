use crate::common::macro_args::MacroArgs;
use crate::common::no_cache_fn::NoCacheFn;
use cache_fn::CacheFn;
use prime_fn::PrimeFn;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::token::Async;
use syn::{parse_str, Expr, GenericArgument, ItemFn, PathArguments, ReturnType, Type};
use ty::CacheType;

pub mod cache_fn;
pub mod prime_fn;
pub mod ty;

pub(crate) fn kash(input: &ItemFn, args: &MacroArgs) -> TokenStream {
    let no_cache_fn = NoCacheFn::new(&input);
    let prime_fn = PrimeFn::new(&input, &args);
    let cache_fn = CacheFn::new(&input, &args);
    let cache_type = CacheType::new(&input, &args);

    quote! {
        #cache_type
        #no_cache_fn
        #prime_fn
        #cache_fn
    }
    .into()
}

fn gen_return_cache_block(result: bool, option: bool) -> TokenStream2 {
    match (result, option) {
        (false, false) => {
            quote! { return Ok(result.to_owned()) }
        }
        (true, false) => {
            quote! { return Ok(result.to_owned()) }
        }
        (false, true) => {
            quote! { return Ok(Some(result.clone())) }
        }
        _ => panic!("the result and option attributes are mutually exclusive"),
    }
}

fn gen_set_cache_block(result: bool, option: bool) -> TokenStream2 {
    let cache_let = match (result, option) {
        (false, false) => {
            quote! { if let Ok(result) = &result  }
        }
        (true, false) => {
            quote! {
                if let Ok(result) = &result
            }
        }
        (false, true) => {
            quote! {
                if let Ok(Some(result)) = &result
            }
        }
        _ => panic!("the result and option attributes are mutually exclusive"),
    };

    quote! {
        #cache_let {
            cache.set(key, result.clone())?;
        }
    }
}

fn gen_cache_ty(
    args: &MacroArgs,
    cache_value_ty: TokenStream2,
    cache_key_ty: TokenStream2,
) -> TokenStream2 {
    let cache_key_ty = args.key.clone().unwrap_or(cache_key_ty.to_string());
    let cache_key_ty = parse_str::<Type>(&cache_key_ty).expect("unable to parse a cache key type");

    // https://github.com/spacejam/sled?tab=readme-ov-file#interaction-with-async
    quote! { kash::DiskCache<#cache_key_ty, #cache_value_ty> }
}

fn gen_cache_create(args: &MacroArgs, cache_name: String) -> TokenStream2 {
    let ttl = &args.ttl;
    let args = args.disk.as_ref().expect("We are in the disk section");

    let connection_config = match &args.connection_config {
        Some(connection_config) => {
            let connection_config = parse_str::<Expr>(connection_config)
                .expect("unable to parse connection_config block");
            Some(quote! { #connection_config })
        }
        None => None,
    };
    let sync_to_disk_on_cache_change = &args.sync_to_disk_on_cache_change;
    let mut create = quote! {
        kash::DiskCache::new(#cache_name)
            .set_sync_to_disk_on_cache_change(#sync_to_disk_on_cache_change)
    };
    if let Some(ttl) = ttl {
        let ttl = parse_str::<Expr>(ttl).expect("Unable to parse ttl");
        create = quote! {
            (#create).set_ttl(#ttl)
        };
    };
    if let Some(connection_config) = connection_config {
        create = quote! {
            (#create).set_connection_config(#connection_config)
        };
    };
    if let Some(dir) = &args.dir {
        create = quote! { (#create).set_disk_directory(#dir) };
    }
    quote! { (#create).build().expect("error constructing DiskCache in #[kash] macro") }
}

fn gen_set_return_block(
    asyncness: &Option<Async>,
    init_cache_ident: TokenStream2,
    function_call: TokenStream2,
    set_cache_and_return: TokenStream2,
) -> TokenStream2 {
    if asyncness.is_some() {
        quote! {
            #function_call
            let cache = #init_cache_ident.get_or_init(init).await;
            #set_cache_and_return
        }
    } else {
        quote! {
            #function_call
            let cache = #init_cache_ident;
            #set_cache_and_return
        }
    }
}

fn gen_use_trait() -> TokenStream2 {
    quote! { use kash::IOKash; }
}

fn gen_cache_value_type(result: bool, option: bool, output: &ReturnType) -> TokenStream2 {
    match output {
        ReturnType::Default => panic!("Should return a Result"),
        ReturnType::Type(_, ty) => match (result, option) {
            (true, true) => panic!("The result and option attributes are mutually exclusive"),
            (false, true) => match output {
                ReturnType::Default => {
                    panic!("Function must return something for result or option attributes")
                }
                ReturnType::Type(_, ty) => {
                    if let Type::Path(typepath) = *ty.clone() {
                        let segments = typepath.path.segments;
                        if let PathArguments::AngleBracketed(brackets) =
                            &segments.last().unwrap().arguments
                        {
                            let inner_ty = brackets.args.first().unwrap();
                            if let GenericArgument::Type(inner_inner_ty) = inner_ty {
                                if let Type::Path(typepath) = inner_inner_ty.clone() {
                                    let segments = typepath.path.segments;
                                    if let PathArguments::AngleBracketed(brackets) =
                                        &segments.last().unwrap().arguments
                                    {
                                        let inner_ty = brackets.args.first().unwrap();
                                        quote! {#inner_ty}
                                    } else {
                                        panic!("Function return type has no inner type")
                                    }
                                } else {
                                    panic!("Function return type is too complex")
                                }
                            } else {
                                panic!("Function return type is too complex")
                            }
                        } else {
                            panic!("Function return type has no inner type")
                        }
                    } else {
                        panic!("Function return type is too complex")
                    }
                }
            },
            _ => match output {
                ReturnType::Default => quote! {#ty},
                ReturnType::Type(_, ty) => {
                    if let Type::Path(typepath) = *ty.clone() {
                        let segments = typepath.path.segments;
                        if let PathArguments::AngleBracketed(brackets) =
                            &segments.last().unwrap().arguments
                        {
                            let inner_ty = brackets.args.first().unwrap();
                            quote! {#inner_ty}
                        } else {
                            panic!("Function return type has no inner type")
                        }
                    } else {
                        panic!("Function return type is too complex")
                    }
                }
            },
        },
    }
}
