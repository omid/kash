use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::token::Async;

pub fn gen_init_and_get(asyncness: &Option<Async>, init_cache_ident: &TokenStream, return_cache_block: TokenStream, async_cache_get_return: TokenStream) -> TokenStream {
    if asyncness.is_some() {
        quote! {
                let cache = #init_cache_ident.get_or_init(init).await;
                #async_cache_get_return
            }
    } else {
        quote! {
                let cache = #init_cache_ident;
                if let Some(result) = cache.get(&key)? {
                    #return_cache_block
                }
            }
    }
}

pub fn gen_function_call(asyncness: &Option<Async>, without_self_names: &[TokenStream], call_prefix: TokenStream, no_cache_fn_ident: Ident) -> TokenStream {
    if asyncness.is_some() {
        quote! {
                let result = #call_prefix #no_cache_fn_ident(#(#without_self_names),*).await;
            }
    } else {
        quote! {
                let result = #call_prefix #no_cache_fn_ident(#(#without_self_names),*);
            }
    }
}
