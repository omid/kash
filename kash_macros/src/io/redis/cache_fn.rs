use crate::common::macro_args::MacroArgs;
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::io::common::{
    gen_function_call, gen_init_and_get, gen_return_cache_block, gen_set_return_block,
};
use crate::io::redis::{gen_cache_create, gen_set_cache_block, gen_use_trait};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ItemFn};

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
        let asyncness = &signature.asyncness;
        let cache_fn_ident_doc = format!("Caches the function [`{}`].", fn_ident);
        let attributes = &self.input.attrs;
        let visibility = &self.input.vis;
        let inputs = &self.input.sig.inputs;

        let (_, without_self_types) = get_input_types(inputs);
        let (_, without_self_names) = get_input_names(inputs);

        let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", fn_ident), fn_ident.span());
        let cache_ident = gen_cache_ident(&self.args.name, fn_ident);

        let call_prefix = if self.args.in_impl {
            quote! { Self:: }
        } else {
            quote! {}
        };
        let no_cache_fn_ident = Ident::new(&format!("{}_no_cache", fn_ident), fn_ident.span());

        let init_cache_ident = if self.args.in_impl {
            quote! {
                &#call_prefix #fn_cache_ident()
            }
        } else {
            quote! {
                &#call_prefix #cache_ident
            }
        };

        let (_, key_expr) =
            make_cache_key_type(&self.args.key, without_self_types, &without_self_names);

        let set_cache_block = gen_set_cache_block(self.args.result, self.args.option, asyncness);
        let return_cache_block = gen_return_cache_block(self.args.result, self.args.option);

        let cache_create = gen_cache_create(self.args, asyncness, &cache_ident);

        let init = if asyncness.is_some() {
            quote! { let kash_init = || async { #cache_create }; }
        } else {
            quote! {}
        };
        let use_trait = gen_use_trait(asyncness);
        let async_cache_get_return = if asyncness.is_some() {
            quote! {
                if let Some(kash_result) = kash_cache.get(&kash_key).await? {
                    #return_cache_block
                }
            }
        } else {
            quote! {
                if let Some(kash_result) = kash_cache.get(&kash_key)? {
                    #return_cache_block
                }
            }
        };
        let init_and_get = gen_init_and_get(
            asyncness,
            &init_cache_ident,
            return_cache_block,
            async_cache_get_return,
        );
        let set_cache_and_return = quote! {
            #set_cache_block
            kash_result
        };
        let function_call = gen_function_call(
            asyncness,
            &without_self_names,
            call_prefix,
            no_cache_fn_ident,
        );

        let do_set_return_block = gen_set_return_block(
            asyncness,
            init_cache_ident,
            function_call,
            set_cache_and_return,
        );

        let expanded = quote! {
            #[doc = #cache_fn_ident_doc]
            #(#attributes)*
             #visibility #signature {
                 #init
                 #use_trait
                 let kash_key = #key_expr;
                 {
                     #init_and_get
                 }
                 #do_set_return_block
             }
        };

        tokens.extend(expanded);
    }
}
