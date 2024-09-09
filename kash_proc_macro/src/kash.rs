use crate::helpers::*;
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident, ItemFn, ReturnType};

#[derive(FromMeta)]
struct MacroArgs {
    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    size: Option<usize>,
    #[darling(default)]
    ttl: Option<u64>,
    #[darling(default)]
    convert: Option<String>,
    #[darling(default)]
    ty: Option<String>,
    #[darling(default)]
    result: bool,
    #[darling(default)]
    option: bool,
    #[darling(default)]
    sync_writes: bool,
    #[darling(default)]
    wrap_return: bool,
    #[darling(default)]
    result_fallback: bool,
    #[darling(default)]
    in_impl: bool,
}

pub fn kash(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    let args = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    let input = parse_macro_input!(input as ItemFn);

    // pull out the parts of the input
    let mut attributes = input.attrs;
    let visibility = input.vis;
    let signature = input.sig;
    let body = input.block;

    // pull out the parts of the function signature
    let fn_ident = signature.ident.clone();
    let inputs = signature.inputs.clone();
    let output = signature.output.clone();
    let asyncness = signature.asyncness;
    let generics = signature.generics.clone();

    let input_tys = get_input_types(&inputs);
    let input_names = get_input_names(&inputs);

    // pull out the output type
    let output_ty = match &output {
        ReturnType::Default => quote! {()},
        ReturnType::Type(_, ty) => quote! {#ty},
    };

    let output_span = output_ty.span();
    let output_ts = TokenStream::from(output_ty.clone());
    let output_parts = get_output_parts(&output_ts);
    let output_string = output_parts.join("::");
    let output_type_display = output_ts.to_string().replace(' ', "");

    if check_wrap_return(args.wrap_return, output_string) {
        return wrap_return_error(output_span, output_type_display);
    }

    let cache_value_ty = find_value_type(args.result, args.option, &output, output_ty);

    let (cache_key_ty, key_convert_block) =
        make_cache_key_type(&args.convert, &args.ty, input_tys, &input_names);

    let cache_ty = quote! {kash::MemoryCache<#cache_key_ty, #cache_value_ty>};

    let cache_ident = match args.name {
        Some(ref name) => Ident::new(name, fn_ident.span()),
        None => Ident::new(&fn_ident.to_string().to_uppercase(), fn_ident.span()),
    };

    let size = if let Some(size) = args.size {
        quote! { .max_capacity(#size) }
    } else {
        quote! {}
    };

    let ttl = if let Some(ttl) = args.ttl {
        quote! { .time_to_live(#ttl) }
    } else {
        quote! {}
    };

    let name = if let Some(ref name) = args.name {
        quote! { .name(#name) }
    } else {
        quote! {}
    };

    let cache_init = quote! {
        static #cache_ident: #cache_ty = #cache_ty::new(::moka::sync::Cache::builder()
            #size
            #ttl
            #name
            .build());
    };

    // make the set cache and return cache blocks
    let (set_cache_block, return_cache_block) = match (&args.result, &args.option) {
        (false, false) => {
            let set_cache_block = quote! { cache.cache_set(key, result.clone()); };
            let return_cache_block = if args.wrap_return {
                quote! { let mut r = result.to_owned(); r.was_cached = true; return r }
            } else {
                quote! { return result.to_owned() }
            };
            (set_cache_block, return_cache_block)
        }
        (true, false) => {
            let set_cache_block = quote! {
                if let Ok(result) = &result {
                    cache.cache_set(key, result.clone());
                }
            };
            let return_cache_block = if args.wrap_return {
                quote! { let mut r = result.to_owned(); r.was_cached = true; return Ok(r) }
            } else {
                quote! { return Ok(result.to_owned()) }
            };
            (set_cache_block, return_cache_block)
        }
        (false, true) => {
            let set_cache_block = quote! {
                if let Some(result) = &result {
                    cache.cache_set(key, result.clone());
                }
            };
            let return_cache_block = if args.wrap_return {
                quote! { let mut r = result.to_owned(); r.was_cached = true; return Some(r) }
            } else {
                quote! { return Some(result.clone()) }
            };
            (set_cache_block, return_cache_block)
        }
        _ => panic!("the result and option attributes are mutually exclusive"),
    };

    if args.result_fallback && args.sync_writes {
        panic!("the result_fallback and sync_writes attributes are mutually exclusive");
    }

    let set_cache_and_return = quote! {
        #set_cache_block
        result
    };

    let no_cache_fn_ident = Ident::new(&format!("{}_no_cache", &fn_ident), fn_ident.span());
    let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", &fn_ident), fn_ident.span());

    let call_prefix = if args.in_impl {
        quote! { Self:: }
    } else {
        quote! {}
    };

    let (may_async, may_await) = if asyncness.is_some() {
        (quote! { async }, quote! { .await })
    } else {
        (quote! {}, quote! {})
    };

    let function_no_cache = quote! {
        #may_async fn #no_cache_fn_ident #generics (#inputs) #output #body
    };

    let function_call = quote! {
        let result = #call_prefix #no_cache_fn_ident(#(#input_names),*) #may_await;
    };

    let cache_ty = if args.in_impl {
        quote! {
            #visibility fn #fn_cache_ident() -> &'static #cache_ty {
                #cache_init;
                &#cache_ident
            }
        }
    } else {
        quote! {
            #visibility #cache_init;
        }
    };

    let prime_do_set_return_block = quote! {
        // run the function and cache the result
        #function_call
        #set_cache_and_return
    };

    let do_set_return_block = if args.sync_writes {
        quote! {
            if let Some(result) = cache.cache_get(&key) {
                #return_cache_block
            }
            #function_call
            #set_cache_and_return
        }
    } else if args.result_fallback {
        quote! {
            let old_val = {
                let (result, has_expired) = cache.cache_get_expired(&key);
                if let (Some(result), false) = (&result, has_expired) {
                    #return_cache_block
                }
                result
            };
            #function_call
            let result = match (result.is_err(), old_val) {
                (true, Some(old_val)) => {
                    Ok(old_val)
                }
                _ => result
            };
            #set_cache_and_return
        }
    } else {
        quote! {
            {
                if let Some(result) = cache.cache_get(&key) {
                    #return_cache_block
                }
            }
            #function_call
            #set_cache_and_return
        }
    };

    let signature_no_muts = get_mut_signature(signature);

    // create a signature for the cache-priming function
    let prime_fn_ident = Ident::new(&format!("{}_prime_cache", &fn_ident), fn_ident.span());
    let mut prime_sig = signature_no_muts.clone();
    prime_sig.ident = prime_fn_ident;

    // make kash static, kash function and prime kash function doc comments
    let cache_ident_doc = format!("Kash static for the [`{}`] function.", fn_ident);
    let no_cache_fn_indent_doc = format!("Origin of the kash function [`{}`].", fn_ident);
    let prime_fn_indent_doc = format!("Primes the kash function [`{}`].", fn_ident);
    let cache_fn_doc_extra = format!(
        "This is a kash function that uses the [`{}`] kash static.",
        cache_ident
    );
    fill_in_attributes(&mut attributes, cache_fn_doc_extra);

    // put it all together
    let expanded = quote! {
        // Kash static
        #[doc = #cache_ident_doc]
        #cache_ty
        // No cache function (origin of the kash function)
        #[doc = #no_cache_fn_indent_doc]
        #(#attributes)*
        #visibility #function_no_cache
        // Kash function
        #(#attributes)*
        #visibility #signature_no_muts {
            use kash::Kash;
            let key = #key_convert_block;
            #do_set_return_block
        }
        // Prime kash function
        #[doc = #prime_fn_indent_doc]
        #[allow(dead_code)]
        #(#attributes)*
        #visibility #prime_sig {
            use kash::Kash;
            let key = #key_convert_block;
            #prime_do_set_return_block
        }
    };

    expanded.into()
}
