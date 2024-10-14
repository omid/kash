use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_str, Expr, Ident, ItemFn, ReturnType};

use crate::common::{
    find_value_type, gen_cache_ident, get_input_names, get_input_types, make_cache_key_type,
};

use super::macro_args::MacroArgs;

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
        let fn_ident = &signature.ident;
        let inputs = &signature.inputs;
        let generics = &signature.generics;
        let output = &signature.output;

        let cache_ident = gen_cache_ident(&self.args.name, fn_ident);
        let moka_ty = if self.input.sig.asyncness.is_some() {
            quote! {::kash::moka::future::Cache}
        } else {
            quote! {::kash::moka::sync::Cache}
        };
        let (_, without_self_types) = get_input_types(inputs);
        let (_, without_self_names) = get_input_names(inputs);

        let (key, _) = make_cache_key_type(
            &self.args.convert,
            &self.args.key,
            without_self_types,
            &without_self_names,
        );
        let output_ty = match &output {
            ReturnType::Default => quote! {()},
            ReturnType::Type(_, key) => quote! {#key},
        };
        let cache_value_ty = find_value_type(self.args.result, self.args.option, output, output_ty);

        let cache_ty = quote! {#moka_ty<#key, #cache_value_ty>};

        let size = if let Some(ref size) = self.args.size {
            let size = parse_str::<Expr>(size).expect("Unable to parse size");
            quote! { .max_capacity(#size) }
        } else {
            quote! {}
        };

        let ttl = if let Some(ref ttl) = self.args.ttl {
            let ttl = parse_str::<Expr>(ttl).expect("Unable to parse ttl");
            quote! { .time_to_live(core::time::Duration::from_secs(#ttl)) }
        } else {
            quote! {}
        };

        let name = if let Some(ref name) = self.args.name {
            quote! { .name(#name) }
        } else {
            quote! {}
        };

        let cache_init = quote! {
            static #cache_ident: ::kash::once_cell::sync::Lazy<#cache_ty> = ::kash::once_cell::sync::Lazy::new(|| {
                #moka_ty::builder()
                    #size
                    #ttl
                    #name
                    .eviction_policy(::kash::moka::policy::EvictionPolicy::lru())
                    .build()
            });
        };
        let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", fn_ident), fn_ident.span());

        let cache_ty = if self.args.in_impl {
            quote! {
                #visibility fn #fn_cache_ident #generics () -> &'static ::kash::once_cell::sync::Lazy<#cache_ty> {
                    #cache_init
                    &#cache_ident
                }
            }
        } else {
            quote! {
                #visibility #cache_init
            }
        };
        let cache_ident_doc = format!("Kash static for the [`{}`] function.", fn_ident);

        let cache_ty = quote! {
            #[doc = #cache_ident_doc]
            #cache_ty
        };
        tokens.extend(cache_ty);
    }
}
