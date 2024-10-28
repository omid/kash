use crate::common::macro_args::MacroArgs;
use crate::common::no_cache_fn::NoCacheFn;
use cache_fn::CacheFn;
use prime_fn::PrimeFn;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_str, Expr, ItemFn, Type};
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

fn gen_set_cache_block(result: bool, option: bool) -> TokenStream2 {
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

    quote! {
        #cache_let {
            kash_cache.set(kash_key, kash_result.clone())?;
        }
    }
}

fn gen_cache_ty(
    args: &MacroArgs,
    cache_value_ty: TokenStream2,
    cache_key_ty: TokenStream2,
) -> TokenStream2 {
    let cache_key_ty = match &args.key {
        None => cache_key_ty.to_string(),
        Some(v) => v.ty.clone(),
    };
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

fn gen_use_trait() -> TokenStream2 {
    quote! { use kash::IOKash; }
}
