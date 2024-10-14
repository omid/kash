use crate::{common::no_cache_fn::NoCacheFn, io_kash::macro_args::MacroArgs};
use cache_fn::CacheFn;
use prime_fn::PrimeFn;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::token::Async;
use syn::{parse_macro_input, parse_str, Block, Expr, ExprClosure, ItemFn, Type};
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

    if let Some(error) = args.validate(&input) {
        return error;
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

fn gen_return_cache_block(wrap_return: bool) -> TokenStream2 {
    if wrap_return {
        quote! { let mut r = ::kash::Return::new(result.clone()); r.was_cached = true; return Ok(r) }
    } else {
        quote! { return Ok(result.clone()) }
    }
}

fn gen_set_cache_block(
    wrap_return: bool,
    disk: bool,
    asyncness: &Option<Async>,
    map_error: &ExprClosure,
) -> TokenStream2 {
    if wrap_return {
        if asyncness.is_some() && !disk {
            quote! {
                if let Ok(result) = &result {
                    cache.set(key, result.value.clone()).await.map_err(#map_error)?;
                }
            }
        } else {
            quote! {
                if let Ok(result) = &result {
                    cache.set(key, result.value.clone()).map_err(#map_error)?;
                }
            }
        }
    } else if asyncness.is_some() && !disk {
        quote! {
            if let Ok(result) = &result {
                cache.set(key, result.clone()).await.map_err(#map_error)?;
            }
        }
    } else {
        quote! {
            if let Ok(result) = &result {
                cache.set(key, result.clone()).map_err(#map_error)?;
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
        (true, false) => {
            if asyncness.is_some() {
                quote! { kash::AsyncRedisCache<#cache_key_ty, #cache_value_ty> }
            } else {
                quote! { kash::RedisCache<#cache_key_ty, #cache_value_ty> }
            }
        }
        // disk
        (false, true) => {
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
    match (
        &args.redis,
        &args.disk,
        &args.ttl,
        &args.cache_prefix_block,
        &args.sync_to_disk_on_cache_change,
        &args.connection_config,
    ) {
        // redis
        (true, false, time, cache_prefix, _, _) => {
            let time = match time {
                Some(time) => quote! { Some(#time) },
                None => quote! { None },
            };

            let cache_prefix = if let Some(cp) = cache_prefix {
                cp.to_string()
            } else {
                format!(" {{ \"kash::io_kash::{}\" }}", cache_ident)
            };
            let cache_prefix =
                parse_str::<Block>(&cache_prefix).expect("unable to parse cache_prefix_block");

            if asyncness.is_some() {
                quote! { kash::AsyncRedisCache::new(#cache_prefix, #time).build().await.expect("error constructing AsyncRedisCache in #[io_kash] macro") }
            } else {
                quote! {
                    kash::RedisCache::new(#cache_prefix, #time).build().expect("error constructing RedisCache in #[io_kash] macro")
                }
            }
        }
        // disk
        (false, true, time, _, sync_to_disk_on_cache_change, connection_config) => {
            let connection_config = match connection_config {
                Some(connection_config) => {
                    let connection_config = parse_str::<Expr>(connection_config)
                        .expect("unable to parse connection_config block");
                    Some(quote! { #connection_config })
                }
                None => None,
            };
            let mut create = quote! {
                kash::DiskCache::new(#cache_name)
            };
            if let Some(time) = time {
                create = quote! {
                    (#create).set_ttl(#time)
                };
            };
            if let Some(sync_to_disk_on_cache_change) = sync_to_disk_on_cache_change {
                create = quote! {
                    (#create).set_sync_to_disk_on_cache_change(#sync_to_disk_on_cache_change)
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

fn gen_use_trait(asyncness: &Option<Async>, disk: bool) -> TokenStream2 {
    if asyncness.is_some() && !disk {
        quote! { use kash::IOKashAsync; }
    } else {
        quote! { use kash::IOKash; }
    }
}
