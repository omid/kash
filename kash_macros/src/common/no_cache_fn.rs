use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Ident, ItemFn};

pub struct NoCacheFn<'a> {
    input: &'a ItemFn,
}

impl<'a> NoCacheFn<'a> {
    pub fn new(input: &'a ItemFn) -> Self {
        Self { input }
    }
}

impl ToTokens for NoCacheFn<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fn_ident = self.input.sig.ident.clone();
        let no_cache_fn_ident = Ident::new(&format!("{}_no_cache", fn_ident), fn_ident.span());

        let no_cache_fn_ident_doc = format!("Origin of the function [`{}`].", no_cache_fn_ident);
        let mut no_cache_fn = self.input.clone();
        no_cache_fn.sig.ident = no_cache_fn_ident;

        let expanded = quote! {
            #[doc = #no_cache_fn_ident_doc]
            #no_cache_fn
        };

        tokens.extend(expanded);
    }
}
