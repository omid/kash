use crate::functions::cache_fn::CacheFn;
use crate::functions::macro_args::MacroArgs;
use crate::functions::no_cache_fn::NoCacheFn;
use crate::functions::prime_fn::PrimeFn;
use crate::functions::ty::CacheType;
use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

pub fn kash(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    let args = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let input = parse_macro_input!(input as ItemFn);

    let no_cache_fn = NoCacheFn::new(&input);
    let prime_fn = PrimeFn::new(&input, &args);
    let cache_fn = CacheFn::new(&input, &args);
    let cache_type = CacheType::new(&input, &args);

    if let Some(error) = args.validate(&input) {
        return error;
    }

    // put it all together
    let expanded = quote! {
        #cache_type
        #no_cache_fn
        #prime_fn
        #cache_fn
    };

    expanded.into()
}
