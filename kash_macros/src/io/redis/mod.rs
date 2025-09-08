use crate::common::macro_args::MacroArgs;
use crate::common::no_cache_fn::NoCacheFn;
use cache_fn::CacheFn;
use prime_fn::PrimeFn;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::token::Async;
use syn::{Block, Expr, ItemFn, Type, parse_str};
use ty::CacheType;

pub mod cache_fn;
pub mod prime_fn;
pub mod ty;

pub(crate) fn kash(input: &ItemFn, args: &MacroArgs) -> TokenStream {
    let no_cache_fn = NoCacheFn::new(input);
    let prime_fn = PrimeFn::new(input, args);
    let cache_fn = CacheFn::new(input, args);
    let cache_type = CacheType::new(input, args);

    quote! {
        #cache_type
        #no_cache_fn
        #prime_fn
        #cache_fn
    }
    .into()
}

fn gen_set_cache_block(result: bool, option: bool, asyncness: &Option<Async>) -> TokenStream2 {
    let cache_let = match (result, option) {
        (false, false) => {
            quote! { if let Ok(kash_result) = &kash_result  }
        }
        (true, false) => {
            quote! {
                if let Ok(kash_result) = &kash_result
            }
        }
        (false, true) => {
            quote! {
                if let Ok(Some(kash_result)) = &kash_result
            }
        }
        _ => unreachable!("All errors should be handled in the `MacroArgs` validation methods"),
    };

    if asyncness.is_some() {
        let may_await = if asyncness.is_some() {
            quote! { .await }
        } else {
            quote! {}
        };

        quote! {
            #cache_let {
                kash_cache.set(kash_key, kash_result.clone())#may_await?;
            }
        }
    } else {
        quote! {
            #cache_let {
                kash_cache.set(kash_key, kash_result.clone())?;
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
    let cache_key_ty = match &args.key {
        None => cache_key_ty.to_string(),
        Some(v) => v.ty.clone(),
    };
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

    let cache_prefix = if let Some(cp) = &args.prefix_block {
        cp.to_string()
    } else {
        format!(" {{ \"{}\" }}", cache_ident)
    };
    let cache_prefix = parse_str::<Block>(&cache_prefix).expect("unable to parse prefix_block");

    if asyncness.is_some() {
        quote! { kash::AsyncRedisCache::new(#cache_prefix, #ttl).build().await.expect("error constructing AsyncRedisCache in #[kash] macro") }
    } else {
        quote! {
            kash::RedisCache::new(#cache_prefix, #ttl).build().expect("error constructing RedisCache in #[kash] macro")
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
