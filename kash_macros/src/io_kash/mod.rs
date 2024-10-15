use crate::io_kash::macro_args::DiskArgs;
use crate::{common::no_cache_fn::NoCacheFn, io_kash::macro_args::MacroArgs};
use cache_fn::CacheFn;
use prime_fn::PrimeFn;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::token::Async;
use syn::{
    parse_macro_input, parse_str, Block, Expr, GenericArgument, ItemFn, PathArguments, ReturnType,
    Type,
};
use ty::CacheType;

pub mod cache_fn;
pub mod macro_args;
pub mod prime_fn;
pub mod ty;

pub fn io_kash(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match MacroArgs::try_from(args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };

    let input = parse_macro_input!(input as ItemFn);

    match args.validate(&input).map_err(|e| e.write_errors()) {
        Ok(_) => {}
        Err(e) => return e.into(),
    }

    let no_cache_fn = NoCacheFn::new(&input);
    let prime_fn = PrimeFn::new(&input, &args);
    let cache_fn = CacheFn::new(&input, &args);
    let cache_type = CacheType::new(&input, &args);

    // put it all together
    let expanded = quote! {
        #cache_type
        #no_cache_fn
        #prime_fn
        #cache_fn
    };

    expanded.into()
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

fn gen_set_cache_block(
    result: bool,
    option: bool,
    disk: &Option<DiskArgs>,
    asyncness: &Option<Async>,
) -> TokenStream2 {
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

    if asyncness.is_some() && disk.is_none() {
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

    match (&args.redis, &args.disk) {
        // redis
        (Some(_), None) => {
            if asyncness.is_some() {
                quote! { kash::AsyncRedisCache<#cache_key_ty, #cache_value_ty> }
            } else {
                quote! { kash::RedisCache<#cache_key_ty, #cache_value_ty> }
            }
        }
        // disk
        (None, Some(_)) => {
            // https://github.com/spacejam/sled?tab=readme-ov-file#interaction-with-async
            quote! { kash::DiskCache<#cache_key_ty, #cache_value_ty> }
        }
        _ => panic!("#[io_kash] cache types could not be determined"),
    }
}

fn gen_cache_create(
    args: &MacroArgs,
    asyncness: &Option<Async>,
    cache_ident: &Ident,
    cache_name: String,
) -> TokenStream2 {
    // make the cache type and create statement
    match (&args.redis, &args.disk, &args.ttl) {
        // redis
        (Some(args), None, ttl) => {
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
                format!(" {{ \"kash::io_kash::{}\" }}", cache_ident)
            };
            let cache_prefix =
                parse_str::<Block>(&cache_prefix).expect("unable to parse cache_prefix_block");

            if asyncness.is_some() {
                quote! { kash::AsyncRedisCache::new(#cache_prefix, #ttl).build().await.expect("error constructing AsyncRedisCache in #[io_kash] macro") }
            } else {
                quote! {
                    kash::RedisCache::new(#cache_prefix, #ttl).build().expect("error constructing RedisCache in #[io_kash] macro")
                }
            }
        }
        // disk
        (None, Some(args), ttl) => {
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
            if let Some(disk_dir) = &args.disk_dir {
                create = quote! { (#create).set_disk_directory(#disk_dir) };
            }
            quote! { (#create).build().expect("error constructing DiskCache in #[io_kash] macro") }
        }
        _ => panic!("#[io_kash] cache types could not be determined"),
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

fn gen_use_trait(asyncness: &Option<Async>, disk: &Option<DiskArgs>) -> TokenStream2 {
    if asyncness.is_some() && disk.is_none() {
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
