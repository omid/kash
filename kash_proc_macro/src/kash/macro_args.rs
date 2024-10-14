use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, ItemFn, ReturnType};

use crate::common::{check_wrap_return, get_output_parts, wrap_return_error};

#[derive(FromMeta, Clone, Debug)]
pub struct MacroArgs {
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub size: Option<String>,
    #[darling(default)]
    pub ttl: Option<String>,
    #[darling(default)]
    pub convert: Option<String>,
    #[darling(default)]
    pub key: Option<String>,
    #[darling(default)]
    pub result: bool,
    #[darling(default)]
    pub option: bool,
    #[darling(default)]
    pub sync_writes: bool,
    #[darling(default)]
    pub wrap_return: bool,
    #[darling(default)]
    pub in_impl: bool,
}

impl MacroArgs {
    pub fn try_from(args: TokenStream) -> Result<Self, Error> {
        let attr_args = match NestedMeta::parse_meta_list(args.into()) {
            Ok(v) => v,
            Err(e) => {
                return Err(e);
            }
        };
        match Self::from_list(&attr_args) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into()),
        }
    }

    pub fn validate(&self, input: &ItemFn) -> Option<TokenStream> {
        // pull out the output type
        let output_ty = match &input.sig.output {
            ReturnType::Default => quote! {()},
            ReturnType::Type(_, key) => quote! {#key},
        };
        let output_ts = TokenStream::from(output_ty.clone());
        let output_parts = get_output_parts(&output_ts);
        let output_string = output_parts.join("::");

        if check_wrap_return(self.wrap_return, output_string) {
            let output_span = output_ty.span();
            let output_type_display = output_ts.to_string().replace(' ', "");
            Some(wrap_return_error(output_span, output_type_display))
        } else {
            None
        }
    }
}
