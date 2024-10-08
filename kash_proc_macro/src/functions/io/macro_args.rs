use crate::helpers::*;
use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::ItemFn;
use syn::{GenericArgument, PathArguments, ReturnType, Type};

#[derive(FromMeta, Clone, Debug)]
pub struct MacroArgs {
    pub map_error: String,
    #[darling(default)]
    pub disk: bool,
    #[darling(default)]
    pub disk_dir: Option<String>,
    #[darling(default)]
    pub redis: bool,
    #[darling(default)]
    pub cache_prefix_block: Option<String>,
    #[darling(default)]
    pub name: Option<String>,
    #[darling(default)]
    pub time: Option<u64>,
    #[darling(default)]
    pub time_refresh: Option<bool>,
    #[darling(default)]
    pub convert: Option<String>,
    #[darling(default)]
    pub wrap_return: bool,
    #[darling(default)]
    pub ty: Option<String>,
    #[darling(default)]
    pub create: Option<String>,
    #[darling(default)]
    pub sync_to_disk_on_cache_change: Option<bool>,
    #[darling(default)]
    pub connection_config: Option<String>,
    #[darling(default)]
    pub in_impl: bool,
}

impl MacroArgs {
    pub fn validate(&self, input: &ItemFn) -> Option<TokenStream> {
        // pull out the output type
        let output = &input.sig.output;
        let output_ty = match output {
            ReturnType::Default => quote! {()},
            ReturnType::Type(_, ty) => quote! {#ty},
        };

        let output_span = output_ty.span();
        let output_ts = TokenStream::from(output_ty);
        let output_parts = get_output_parts(&output_ts);
        let output_string = output_parts.join("::");
        let output_type_display = output_ts.to_string().replace(' ', "");

        // if `wrap_return`, then enforce that the return type
        // is something wrapped in `Return`. Either `Return<T>` or the
        // fully qualified `kash::Return<T>`
        if self.wrap_return
            && !output_string.contains("Return")
            && !output_string.contains("kash::Return")
        {
            Some(
                syn::Error::new(
                    output_span,
                    format!(
                        "\nWhen specifying `wrap_return`, \
                    the return type must be wrapped in `kash::Return<T>`. \n\
                    The following return types are supported: \n\
                    |    `Result<kash::Return<T>, E>`\n\
                    Found type: {t}.",
                        t = output_type_display
                    ),
                )
                .to_compile_error()
                .into(),
            )
        } else {
            match output.clone() {
                ReturnType::Default => {
                    panic!(
                        "#[io_kash] functions must return `Result`s, found {:?}",
                        output_type_display
                    );
                }
                ReturnType::Type(_, ty) => {
                    if let Type::Path(typepath) = *ty {
                        let segments = typepath.path.segments;
                        if let PathArguments::AngleBracketed(brackets) =
                            &segments.last().unwrap().arguments
                        {
                            let inner_ty = brackets.args.first().unwrap();
                            if output_string.contains("Return")
                                || output_string.contains("kash::Return")
                            {
                                if let GenericArgument::Type(Type::Path(typepath)) = inner_ty {
                                    let segments = &typepath.path.segments;
                                    if let PathArguments::AngleBracketed(_) =
                                        &segments.last().unwrap().arguments
                                    {
                                        None
                                    } else {
                                        panic!(
                                            "#[io_kash] unable to determine cache value type, found {:?}",
                                            output_type_display
                                        );
                                    }
                                } else {
                                    panic!(
                                        "#[io_kash] unable to determine cache value type, found {:?}",
                                        output_type_display
                                    );
                                }
                            } else {
                                None
                            }
                        } else {
                            panic!("#[io_kash] functions must return `Result`s")
                        }
                    } else {
                        panic!(
                    "function return type too complex, #[io_kash] functions must return `Result`s"
                )
                    }
                }
            }
        }
    }
}
