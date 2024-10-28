use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ItemFn};

use crate::common::macro_args::MacroArgs;
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::mem::gen_local_cache;

#[derive(Debug, Clone)]
pub struct CacheFn<'a> {
    input: &'a ItemFn,
    args: &'a MacroArgs,
}

impl<'a> CacheFn<'a> {
    pub fn new(input: &'a ItemFn, args: &'a MacroArgs) -> Self {
        Self { input, args }
    }
}

impl ToTokens for CacheFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let signature = &self.input.sig;
        let fn_ident = &signature.ident;

        let cache_fn_ident_doc = format!("Caches the function [`{}`].", fn_ident);
        let attributes = &self.input.attrs;
        let visibility = &self.input.vis;
        let inputs = &self.input.sig.inputs;

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
        let may_await = if self.input.sig.asyncness.is_some() {
            quote! { .await }
        } else {
            quote! {}
        };
        let mut function_call = quote! {
            #call_prefix #no_cache_fn_ident(#(#maybe_with_self_names),*)
        };

        if self.input.sig.asyncness.is_none() {
            function_call = quote! {
                || #function_call
            }
        }

        let (insert, may_return_early, may_wrap) = match (self.args.result, self.args.option) {
            (false, false) => (quote!(.or_insert_with(#function_call)), quote!(), quote!()),
            (true, false) => (
                quote!(.or_try_insert_with(#function_call)),
                quote!(.map_err(|e| e.deref().clone())?),
                quote!(Ok),
            ),
            (false, true) => (
                quote!(.or_optionally_insert_with(#function_call) ),
                quote!(?),
                quote!(Some),
            ),
            _ => unreachable!("All errors should be handled in the `MacroArgs` validation methods"),
        };

        let do_set_return_block = quote! {
            use std::ops::Deref;
            #may_wrap (#local_cache.entry_by_ref(&#key_expr) #insert #may_await #may_return_early .into_value() .clone())
        };

        let expanded = quote! {
            #[doc = #cache_fn_ident_doc]
            #(#attributes)*
            #visibility #signature {
                #do_set_return_block
            }
        };

        tokens.extend(expanded);
    }
}
