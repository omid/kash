use crate::common::macro_args::MacroArgs;
use crate::common::{gen_cache_ident, get_input_names, get_input_types, make_cache_key_type};
use crate::io::common::gen_set_return_block;
use crate::io::disk::{gen_cache_create, gen_set_cache_block, gen_use_trait};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
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
        let signature = &self.input.sig;
        let fn_ident = &signature.ident;
        let asyncness = &signature.asyncness;
        let prime_fn_ident = Ident::new(&format!("{}_prime_cache", &fn_ident), fn_ident.span());
        let mut prime_sig = signature.clone();
        prime_sig.ident = prime_fn_ident;

        let prime_fn_indent_doc = format!("Primes the function [`{}`].", fn_ident);
        let attributes = &self.input.attrs;
        let visibility = &self.input.vis;
        let inputs = &self.input.sig.inputs;

        let (_, without_self_types) = get_input_types(inputs);
        let (maybe_with_self_names, without_self_names) = get_input_names(inputs);

        let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", fn_ident), fn_ident.span());
        let cache_ident = gen_cache_ident(&self.args.name, fn_ident);

        let call_prefix = if self.args.in_impl {
            quote! { Self:: }
        } else {
            quote! {}
        };
        let no_cache_fn_ident = Ident::new(&format!("{}_no_cache", fn_ident), fn_ident.span());

        let may_await = if asyncness.is_some() {
            quote! {.await}
        } else {
            quote! {}
        };

        let init_cache_ident = if self.args.in_impl {
            quote! {
                &#call_prefix #fn_cache_ident()
            }
        } else {
            quote! {
                &#call_prefix #cache_ident
            }
        };

        let function_call = quote! {
            let kash_result = #call_prefix #no_cache_fn_ident(#(#maybe_with_self_names),*) #may_await;
        };

        let (_, key_expr) =
            make_cache_key_type(&self.args.key, without_self_types, &without_self_names);
        let cache_name = cache_ident.to_string();

        let set_cache_block = gen_set_cache_block(self.args.result, self.args.option);

        let cache_create = gen_cache_create(self.args, cache_name);

        let init = if asyncness.is_some() {
            quote! { let kash_init = || async { #cache_create }; }
        } else {
            quote! {}
        };
        let use_trait = gen_use_trait();
        let set_cache_and_return = quote! {
            #set_cache_block
            kash_result
        };
        let do_set_return_block = gen_set_return_block(
            asyncness,
            init_cache_ident,
            function_call,
            set_cache_and_return,
        );

        let expanded = quote! {
            #[doc = #prime_fn_indent_doc]
            #[allow(dead_code)]
            #(#attributes)*
            #visibility #prime_sig {
                #use_trait
                #init
                let kash_key = #key_expr;
                #do_set_return_block
            }
        };

        tokens.extend(expanded);
    }
}
