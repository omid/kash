use crate::common::macro_args::MacroArgs;
use crate::common::no_cache_fn::NoCacheFn;
use crate::mem::cache_fn::CacheFn;
use crate::mem::prime_fn::PrimeFn;
use crate::mem::ty::CacheType;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{ItemFn, PathArguments, ReturnType, Type};

pub mod cache_fn;
pub mod prime_fn;
pub mod ty;

pub(super) fn kash(input: &ItemFn, args: &MacroArgs) -> TokenStream {
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

fn gen_set_cache_block(
    local_cache: &TokenStream2,
    result: bool,
    option: bool,
    may_await: &TokenStream2,
) -> TokenStream2 {
    match (result, option) {
        (false, false) => {
            quote! { #local_cache.insert(kash_key, kash_result.clone())#may_await; }
        }
        (true, false) => {
            quote! {
                if let Ok(kash_result) = &kash_result {
                    #local_cache.insert(kash_key, kash_result.clone())#may_await;
                }
            }
        }
        (false, true) => {
            quote! {
                if let Some(kash_result) = &kash_result {
                    #local_cache.insert(kash_key, kash_result.clone())#may_await;
                }
            }
        }
        _ => unreachable!("All errors should be handled in the `MacroArgs` validation methods"),
    }
}

// Find the type of the value to store.
// Normally it's the same as the return type of the functions, but
// for Options and Results it's the (first) inner type. So for
// Option<u32>, store u32, for Result<i32, String>, store i32, etc.
fn gen_cache_value_type(result: bool, option: bool, output: &ReturnType) -> TokenStream2 {
    match (result, option) {
        (false, false) => match &output {
            ReturnType::Default => quote! {()},
            ReturnType::Type(_, key) => quote! {#key},
        },
        (true, true) => {
            unreachable!("All errors should be handled in the `MacroArgs` validation methods")
        }
        _ => match output.clone() {
            ReturnType::Default => {
                panic!("Function must return something for `result` or `option` attributes")
            }
            ReturnType::Type(_, ty) => {
                if let Type::Path(typepath) = *ty {
                    let segments = typepath.path.segments;
                    if let PathArguments::AngleBracketed(brackets) =
                        &segments.last().unwrap().arguments
                    {
                        let inner_ty = brackets.args.first().unwrap();
                        quote! {#inner_ty}
                    } else {
                        panic!(
                            "Function return type has no inner type, you should remove `result` or `option`"
                        )
                    }
                } else {
                    panic!("Function return type is too complex")
                }
            }
        },
    }
}

fn gen_local_cache(
    in_impl: bool,
    fn_cache_ident: Ident,
    cache_ident: Ident,
) -> proc_macro2::TokenStream {
    if in_impl {
        quote! {Self:: #fn_cache_ident()}
    } else {
        quote! {#cache_ident}
    }
}
