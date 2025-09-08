use crate::common::macro_args::MacroArgs;
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::mem::{gen_local_cache, gen_set_cache_block};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Ident, ItemFn};

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
        let sig = &self.input.sig;
        let fn_ident = &sig.ident;
        let prime_fn_ident = Ident::new(&format!("{}_prime_cache", fn_ident), fn_ident.span());
        let mut prime_sig = sig.clone();
        prime_sig.ident = prime_fn_ident;

        let prime_fn_indent_doc = format!("Primes the function [`{}`].", fn_ident);
        let attrs = &self.input.attrs;
        let vis = &self.input.vis;
        let inputs = &sig.inputs;

        let (_, without_self_types) = get_input_types(inputs);
        let (maybe_with_self_names, without_self_names) = get_input_names(inputs);

        let (_, key_expr) =
            make_cache_key_type(&self.args.key, without_self_types, &without_self_names);
        let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", fn_ident), fn_ident.span());
        let cache_ident = gen_cache_ident(&self.args.name, fn_ident);

        let local_cache = gen_local_cache(self.args.in_impl, fn_cache_ident, cache_ident);
        let call_prefix = if self.args.in_impl {
            quote! { Self:: }
        } else {
            quote! {}
        };
        let no_cache_fn_ident = Ident::new(&format!("{}_no_cache", fn_ident), fn_ident.span());

        let may_await = if sig.asyncness.is_some() {
            quote! {.await}
        } else {
            quote! {}
        };

        let function_call = quote! {
            let kash_result = #call_prefix #no_cache_fn_ident(#(#maybe_with_self_names),*) #may_await;
        };
        let set_cache_block =
            gen_set_cache_block(&local_cache, self.args.result, self.args.option, &may_await);
        let set_cache_and_return = quote! {
            #set_cache_block
            kash_result
        };
        let prime_do_set_return_block = quote! {
            // run the function and cache the result
            #function_call
            #set_cache_and_return
        };

        let expanded = quote! {
            #[doc = #prime_fn_indent_doc]
            #[allow(dead_code)]
            #(#attrs)*
            #vis #prime_sig {
                let kash_key = #key_expr;
                #prime_do_set_return_block
            }
        };

        tokens.extend(expanded);
    }
}
