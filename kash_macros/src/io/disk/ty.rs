use crate::common::macro_args::MacroArgs;
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::io::common::gen_cache_value_type;
use crate::io::disk::{gen_cache_create, gen_cache_ty};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ItemFn};

// struct for cache function
#[derive(Debug, Clone)]
pub struct CacheType<'a> {
    input: &'a ItemFn,
    args: &'a MacroArgs,
}

impl<'a> CacheType<'a> {
    pub fn new(input: &'a ItemFn, args: &'a MacroArgs) -> Self {
        Self { input, args }
    }
}

impl ToTokens for CacheType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let visibility = &self.input.vis;
        let signature = &self.input.sig;
        let asyncness = &signature.asyncness;
        let fn_ident = &signature.ident;
        let inputs = &signature.inputs;
        let output = &signature.output;

        let cache_ident = gen_cache_ident(&self.args.name, fn_ident);

        let cache_name = cache_ident.to_string();

        let (_, without_self_types) = get_input_types(inputs);
        let (_, without_self_names) = get_input_names(inputs);

        let cache_value_ty = gen_cache_value_type(self.args.result, self.args.option, output);

        let (cache_key_ty, _) = make_cache_key_type(
            &self.args.convert,
            &self.args.key,
            without_self_types,
            &without_self_names,
        );

        let cache_ty = gen_cache_ty(self.args, cache_value_ty, cache_key_ty);
        let cache_create = gen_cache_create(self.args, cache_name);

        let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", fn_ident), fn_ident.span());

        let key = match (asyncness.is_some(), self.args.in_impl) {
            (true, true) => quote! {
                #visibility fn #fn_cache_ident() -> &'static ::kash::async_sync::OnceCell<#cache_ty> {
                    static #cache_ident: ::kash::async_sync::OnceCell<#cache_ty> = ::kash::async_sync::OnceCell::const_new();
                    &#cache_ident
                }
            },
            (true, false) => quote! {
                #visibility static #cache_ident: ::kash::async_sync::OnceCell<#cache_ty> = ::kash::async_sync::OnceCell::const_new();
            },

            (false, true) => quote! {
                #visibility fn #fn_cache_ident() -> &'static ::kash::once_cell::sync::Lazy<#cache_ty> {
                    static #cache_ident: ::kash::once_cell::sync::Lazy<#cache_ty> = ::kash::once_cell::sync::Lazy::new(|| #cache_create);
                    &#cache_ident
                }
            },
            (false, false) => quote! {
                #visibility static #cache_ident: ::kash::once_cell::sync::Lazy<#cache_ty> = ::kash::once_cell::sync::Lazy::new(|| #cache_create);
            },
        };

        let cache_ident_doc = format!("Kash static for the [`{}`] function.", fn_ident);

        let cache_ty = quote! {
            #[doc = #cache_ident_doc]
            #key
        };
        tokens.extend(cache_ty);
    }
}
