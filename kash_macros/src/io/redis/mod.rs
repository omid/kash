use crate::common::macro_args::MacroArgs;
use crate::common::no_cache_fn::NoCacheFn;
use cache_fn::CacheFn;
use prime_fn::PrimeFn;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::token::Async;
use syn::{parse_str, Block, Expr, GenericArgument, ItemFn, PathArguments, ReturnType, Type};
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

fn gen_set_cache_block(result: bool, option: bool, asyncness: &Option<Async>) -> TokenStream2 {
    let may_await = if asyncness.is_some() {
        quote! { .await }
    } else {
        quote! {}
    };

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

    if asyncness.is_some() {
        quote! {
            #cache_let {
                cache.set(key, result.clone())#may_await?;
            }
        }
    } else {
        quote! {
            #cache_let {
                cache.set(key, result.clone())?;
            }
        }
    }
}

fn gen_cache_ty(
    args: &MacroArgs,
    asyncness: &Option<Async>,
    cache_value_ty: TokenStream2,
    cache_key_ty: TokenStream2,
) -> TokenStream2 {
    let cache_key_ty = args.key.clone().unwrap_or(cache_key_ty.to_string());
    let cache_key_ty = parse_str::<Type>(&cache_key_ty).expect("unable to parse a cache key type");

    if asyncness.is_some() {
        quote! { kash::AsyncRedisCache<#cache_key_ty, #cache_value_ty> }
    } else {
        quote! { kash::RedisCache<#cache_key_ty, #cache_value_ty> }
    }
}

fn gen_cache_create(
    args: &MacroArgs,
    asyncness: &Option<Async>,
    cache_ident: &Ident,
) -> TokenStream2 {
    let ttl = &args.ttl;
    let args = args.redis.as_ref().expect("We are in the redis section");

    let ttl = match ttl {
        Some(ttl) => {
            let ttl = parse_str::<Expr>(ttl).expect("Unable to parse ttl");
            quote! { Some(#ttl) }
        }
        None => quote! { None },
    };

    let cache_prefix = if let Some(cp) = &args.cache_prefix_block {
        cp.to_string()
    } else {
        format!(" {{ \"kash::kash::{}\" }}", cache_ident)
    };
    let cache_prefix =
        parse_str::<Block>(&cache_prefix).expect("unable to parse cache_prefix_block");

    if asyncness.is_some() {
        quote! { kash::AsyncRedisCache::new(#cache_prefix, #ttl).build().await.expect("error constructing AsyncRedisCache in #[kash] macro") }
    } else {
        quote! {
            kash::RedisCache::new(#cache_prefix, #ttl).build().expect("error constructing RedisCache in #[kash] macro")
        }
    }
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

fn gen_use_trait(asyncness: &Option<Async>) -> TokenStream2 {
    if asyncness.is_some() {
        quote! { use kash::IOKashAsync; }
    } else {
        quote! { use kash::IOKash; }
    }
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
