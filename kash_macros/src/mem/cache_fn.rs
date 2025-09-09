use crate::common::macro_args::MacroArgs;
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::mem::gen_local_cache;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
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
        let sig = &self.input.sig;
        let fn_ident = &sig.ident;

        let cache_fn_ident_doc = format!("Caches the function [`{}`].", fn_ident);
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
            quote! { .await }
        } else {
            quote! {}
        };
        let function_call = quote! {
            #call_prefix #no_cache_fn_ident(#(#maybe_with_self_names),*) #may_await
        };

        let may_return = if self.args.option {
            quote!(?)
        } else {
            quote!()
        };

        let (insert, may_wrap) = match (self.args.result, self.args.option) {
            (false, false) => (quote!(.or_insert(#function_call) #may_await), quote!()),
            (true, false) => (quote!(.or_insert(#function_call?) #may_await), quote!(Ok)),
            (false, true) => (
                quote!(.or_optionally_insert_with(|| #function_call) #may_await),
                quote!(Some),
            ),
            _ => unreachable!("All errors should be handled in the `MacroArgs` validation methods"),
        };

        let do_set_return_block = quote! {
            let val = #local_cache.get(&#key_expr) #may_await;
            if let Some(val) = val {
                #may_wrap (val)
            } else {
                #may_wrap (#local_cache
                    .entry_by_ref(&#key_expr)
                    #insert
                    #may_return
                    .into_value())
            }
        };

        let expanded = quote! {
            #[doc = #cache_fn_ident_doc]
            #(#attrs)*
            #vis #sig {
                #do_set_return_block
            }
        };

        tokens.extend(expanded);
    }
}
