use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ItemFn};

use crate::common::macro_args::MacroArgs;
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::mem::{gen_local_cache, gen_return_cache_block, gen_set_cache_block};

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

        let (_, convert_block) = make_cache_key_type(
            &self.args.convert,
            &self.args.key,
            without_self_types,
            &without_self_names,
        );
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
        let set_cache_block = gen_set_cache_block(self.args.result, self.args.option, &may_await);
        let return_cache_block = gen_return_cache_block(self.args.result, self.args.option);
        let function_call = quote! {
            let result = #call_prefix #no_cache_fn_ident(#(#maybe_with_self_names),*) #may_await;
        };
        let set_cache_and_return = quote! {
            #set_cache_block
            result
        };
        let do_set_return_block = if self.args.sync_writes {
            quote! {
                #local_cache
                if let Some(result) = cache.get(&key)#may_await {
                    #return_cache_block
                }
                #function_call
                #set_cache_and_return
            }
        } else {
            quote! {
                #local_cache
                {
                    if let Some(result) = cache.get(&key)#may_await {
                        #return_cache_block
                    }
                }
                #function_call
                #set_cache_and_return
            }
        };

        let expanded = quote! {
            #[doc = #cache_fn_ident_doc]
            #(#attributes)*
            #visibility #signature {
                let key = #convert_block;
                #do_set_return_block
            }
        };

        tokens.extend(expanded);
    }
}
