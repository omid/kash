use proc_macro2::{Ident, TokenStream, TokenStream as TokenStream2};
use quote::quote;
use syn::token::Async;
use syn::{GenericArgument, PathArguments, ReturnType, Type};

pub fn gen_init_and_get(
    asyncness: &Option<Async>,
    init_cache_ident: &TokenStream,
    return_cache_block: TokenStream,
    async_cache_get_return: TokenStream,
) -> TokenStream {
    if asyncness.is_some() {
        quote! {
            let kash_cache = #init_cache_ident.get_or_init(kash_init).await;
            #async_cache_get_return
        }
    } else {
        quote! {
            let kash_cache = #init_cache_ident;
            if let Some(kash_result) = kash_cache.get(&kash_key)? {
                #return_cache_block
            }
        }
    }
}

pub fn gen_function_call(
    asyncness: &Option<Async>,
    without_self_names: &[TokenStream],
    call_prefix: TokenStream,
    no_cache_fn_ident: Ident,
) -> TokenStream {
    if asyncness.is_some() {
        quote! {
            let kash_result = #call_prefix #no_cache_fn_ident(#(#without_self_names),*).await;
        }
    } else {
        quote! {
            let kash_result = #call_prefix #no_cache_fn_ident(#(#without_self_names),*);
        }
    }
}

pub fn gen_return_cache_block(result: bool, option: bool) -> TokenStream2 {
    match (result, option) {
        (false, false) => {
            quote! { return Ok(kash_result.to_owned()) }
        }
        (true, false) => {
            quote! { return Ok(kash_result.to_owned()) }
        }
        (false, true) => {
            quote! { return Ok(Some(kash_result.clone())) }
        }
        _ => unreachable!("All errors should be handled in the `MacroArgs` validation methods"),
    }
}

pub fn gen_set_return_block(
    asyncness: &Option<Async>,
    init_cache_ident: TokenStream2,
    function_call: TokenStream2,
    set_cache_and_return: TokenStream2,
) -> TokenStream2 {
    if asyncness.is_some() {
        quote! {
            #function_call
            let kash_cache = #init_cache_ident.get_or_init(kash_init).await;
            #set_cache_and_return
        }
    } else {
        quote! {
            #function_call
            let kash_cache = #init_cache_ident;
            #set_cache_and_return
        }
    }
}

pub fn gen_cache_value_type(result: bool, option: bool, output: &ReturnType) -> TokenStream2 {
    match output {
        ReturnType::Default => panic!("Should return a Result"),
        ReturnType::Type(_, ty) => match (result, option) {
            (true, true) => {
                unreachable!("All errors should be handled in the `MacroArgs` validation methods")
            }
            (false, true) => match output {
                ReturnType::Default => {
                    panic!("Function must return something for `result` or `option` attributes")
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
                                        panic!(
                                            "Function return type has no inner type, you should remove `result` or `option`"
                                        )
                                    }
                                } else {
                                    panic!("Function return type is too complex")
                                }
                            } else {
                                panic!("Function return type is too complex")
                            }
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
                            panic!(
                                "Function return type has no inner type, you should remove `result` or `option`"
                            )
                        }
                    } else {
                        panic!("Function return type is too complex")
                    }
                }
            },
        },
    }
}
