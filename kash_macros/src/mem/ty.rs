use crate::common::macro_args::{EvictionPolicy, MacroArgs};
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::mem::gen_cache_value_type;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Ident, ItemFn, parse_str};

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
        let vis = &self.input.vis;
        let sig = &self.input.sig;
        let fn_ident = &sig.ident;
        let inputs = &sig.inputs;
        let generics = &sig.generics;
        let output = &sig.output;

        let cache_ident = gen_cache_ident(&self.args.name, fn_ident);
        let moka_ty = if sig.asyncness.is_some() {
            quote! {::kash::moka::future::Cache}
        } else {
            quote! {::kash::moka::sync::Cache}
        };
        let (_, without_self_types) = get_input_types(inputs);
        let (_, without_self_names) = get_input_names(inputs);

        let (key, _) = make_cache_key_type(&self.args.key, without_self_types, &without_self_names);

        let cache_value_ty = gen_cache_value_type(self.args.result, self.args.option, output);

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

        let policy = match self.args.eviction_policy.unwrap_or_default() {
            EvictionPolicy::Lfu => quote! { tiny_lfu },
            EvictionPolicy::Lru => quote! { lru },
        };

        let cache_init = quote! {
            static #cache_ident: ::kash::once_cell::sync::Lazy<#cache_ty> = ::kash::once_cell::sync::Lazy::new(|| {
                #moka_ty::builder()
                    #size
                    #ttl
                    #name
                    .eviction_policy(::kash::moka::policy::EvictionPolicy::#policy())
                    .build()
            });
        };
        let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", fn_ident), fn_ident.span());

        let cache_ty = if self.args.in_impl {
            quote! {
                #vis fn #fn_cache_ident #generics () -> &'static ::kash::once_cell::sync::Lazy<#cache_ty> {
                    #cache_init
                    &#cache_ident
                }
            }
        } else {
            quote! {
                #vis #cache_init
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
