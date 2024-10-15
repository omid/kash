use crate::kash::macro_args::MacroArgs;
use crate::kash::prime_fn::PrimeFn;
use crate::kash::ty::CacheType;
use crate::{common::no_cache_fn::NoCacheFn, kash::cache_fn::CacheFn};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, ItemFn, PathArguments, ReturnType, Type};

pub mod cache_fn;
pub mod macro_args;
pub mod prime_fn;
pub mod ty;

pub fn kash(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match MacroArgs::try_from(args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };

    let input = parse_macro_input!(input as ItemFn);

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

fn gen_set_cache_block(result: bool, option: bool, may_await: &TokenStream2) -> TokenStream2 {
    match (result, option) {
        (false, false) => {
            quote! { cache.insert(key, result.clone())#may_await; }
        }
        (true, false) => {
            quote! {
                if let Ok(result) = &result {
                    cache.insert(key, result.clone())#may_await;
                }
            }
        }
        (false, true) => {
            quote! {
                if let Some(result) = &result {
                    cache.insert(key, result.clone())#may_await;
                }
            }
        }
        _ => panic!("the result and option attributes are mutually exclusive"),
    }
}

fn gen_return_cache_block(result: bool, option: bool) -> TokenStream2 {
    match (result, option) {
        (false, false) => {
            quote! { return result.to_owned() }
        }
        (true, false) => {
            quote! { return Ok(result.to_owned()) }
        }
        (false, true) => {
            quote! { return Some(result.clone()) }
        }
        _ => panic!("the result and option attributes are mutually exclusive"),
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
        (true, true) => panic!("The result and option attributes are mutually exclusive"),
        _ => match output.clone() {
            ReturnType::Default => {
                panic!("Function must return something for result or option attributes")
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
                        panic!("Function return type has no inner type")
                    }
                } else {
                    panic!("Function return type is too complex")
                }
            }
        },
    }
}
