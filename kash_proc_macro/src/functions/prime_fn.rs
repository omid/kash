use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ItemFn};

use crate::helpers::{get_input_names, get_input_types, make_cache_key_type};

use super::macro_args::MacroArgs;

// struct for prime function
#[derive(Debug, Clone)]
pub struct PrimeFn<'a> {
    input: &'a ItemFn,
    args: &'a MacroArgs,
}

impl<'a> PrimeFn<'a> {
    pub fn new(input: &'a ItemFn, args: &'a MacroArgs) -> Self {
        Self { input, args }
    }
}

impl ToTokens for PrimeFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let signature = &self.input.sig;
        let fn_ident = &signature.ident;
        let prime_fn_ident = Ident::new(&format!("{}_prime_cache", &fn_ident), fn_ident.span());
        let mut prime_sig = signature.clone();
        prime_sig.ident = prime_fn_ident;

        let prime_fn_indent_doc = format!("Primes the kash function [`{}`].", fn_ident);
        let attributes = &self.input.attrs;
        let visibility = &self.input.vis;
        let inputs = &self.input.sig.inputs;

        let (_, without_self_types) = get_input_types(inputs);
        let (maybe_with_self_names, without_self_names) = get_input_names(inputs);

        let (_, convert_block) = make_cache_key_type(
            &self.args.convert,
            &self.args.key,
            without_self_types,
            &without_self_names,
        );
        let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", &fn_ident), fn_ident.span());
        let cache_ident = match self.args.name {
            Some(ref name) => Ident::new(name, fn_ident.span()),
            None => Ident::new(&fn_ident.to_string().to_uppercase(), fn_ident.span()),
        };
        let local_cache = if self.args.in_impl {
            quote! {let cache = Self:: #fn_cache_ident().clone();}
        } else {
            quote! {let cache = #cache_ident.clone();}
        };
        let call_prefix = if self.args.in_impl {
            quote! { Self:: }
        } else {
            quote! {}
        };
        let no_cache_fn_ident = Ident::new(&format!("{}_no_cache", &fn_ident), fn_ident.span());

        let may_await = if self.input.sig.asyncness.is_some() {
            quote! {.await}
        } else {
            quote! {}
        };

        let function_call = quote! {
            let result = #call_prefix #no_cache_fn_ident(#(#maybe_with_self_names),*) #may_await;
        };
        // make the set cache and return cache blocks
        let (set_cache_block, _) = match (&self.args.result, &self.args.option) {
            (false, false) => {
                let set_cache_block = quote! { cache.insert(key, result.clone())#may_await; };
                let return_cache_block = if self.args.wrap_return {
                    quote! { let mut r = result.to_owned(); r.was_cached = true; return r }
                } else {
                    quote! { return result.to_owned() }
                };
                (set_cache_block, return_cache_block)
            }
            (true, false) => {
                let set_cache_block = quote! {
                    if let Ok(result) = &result {
                        cache.insert(key, result.clone())#may_await;
                    }
                };
                let return_cache_block = if self.args.wrap_return {
                    quote! { let mut r = result.to_owned(); r.was_cached = true; return Ok(r) }
                } else {
                    quote! { return Ok(result.to_owned()) }
                };
                (set_cache_block, return_cache_block)
            }
            (false, true) => {
                let set_cache_block = quote! {
                    if let Some(result) = &result {
                        cache.insert(key, result.clone())#may_await;
                    }
                };
                let return_cache_block = if self.args.wrap_return {
                    quote! { let mut r = result.to_owned(); r.was_cached = true; return Some(r) }
                } else {
                    quote! { return Some(result.clone()) }
                };
                (set_cache_block, return_cache_block)
            }
            _ => panic!("the result and option attributes are mutually exclusive"),
        };
        let set_cache_and_return = quote! {
            #set_cache_block
            result
        };
        let prime_do_set_return_block = quote! {
            #local_cache
            // run the function and cache the result
            #function_call
            #set_cache_and_return
        };

        let expanded = quote! {
            #[doc = #prime_fn_indent_doc]
            #[allow(dead_code)]
            #(#attributes)*
            #visibility #prime_sig {
                let key = #convert_block;
                #prime_do_set_return_block
            }
        };

        tokens.extend(expanded);
    }
}
