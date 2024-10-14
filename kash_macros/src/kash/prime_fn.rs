use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ItemFn};

use super::macro_args::MacroArgs;
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::kash::gen_set_cache_block;

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
        let prime_fn_ident = Ident::new(&format!("{}_prime_cache", fn_ident), fn_ident.span());
        let mut prime_sig = signature.clone();
        prime_sig.ident = prime_fn_ident;

        let prime_fn_indent_doc = format!("Primes the function [`{}`].", fn_ident);
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
        let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", fn_ident), fn_ident.span());
        let cache_ident = gen_cache_ident(&self.args.name, fn_ident);

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
        let no_cache_fn_ident = Ident::new(&format!("{}_no_cache", fn_ident), fn_ident.span());

        let may_await = if self.input.sig.asyncness.is_some() {
            quote! {.await}
        } else {
            quote! {}
        };

        let function_call = quote! {
            let result = #call_prefix #no_cache_fn_ident(#(#maybe_with_self_names),*) #may_await;
        };
        let set_cache_block = gen_set_cache_block(self.args.result, self.args.option, &may_await);
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
