use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, GenericArgument, ItemFn, PathArguments, ReturnType, Type};

use crate::common::get_output_parts;

#[derive(FromMeta, Clone, Debug)]
// #[darling(and_then = "Self::validate")]
pub struct MacroArgs {
    #[darling(default)]
    pub name: Option<String>,
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
    pub in_impl: bool,

    // #[darling(default)]
    // pub size: Option<String>,
    // #[darling(default)]
    // pub sync_writes: bool,
    #[darling(default)]
    pub disk: Option<DiskArgs>,
    #[darling(default)]
    pub redis: Option<RedisArgs>,
}

#[derive(Clone, Debug, Default)]
pub struct RedisArgs {
    pub cache_prefix_block: Option<String>,
}

impl From<RedisArgsHelper> for RedisArgs {
    fn from(value: RedisArgsHelper) -> Self {
        Self {
            cache_prefix_block: value.cache_prefix_block,
        }
    }
}

#[derive(FromMeta)]
struct RedisArgsHelper {
    #[darling(default)]
    pub cache_prefix_block: Option<String>,
}

impl FromMeta for RedisArgs {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let helper = RedisArgsHelper::from_list(items)?;
        Ok(helper.into())
    }

    fn from_word() -> darling::Result<Self> {
        Self::from_list(&[])
    }
}

#[derive(Clone, Debug, Default)]
pub struct DiskArgs {
    pub connection_config: Option<String>,
    pub sync_to_disk_on_cache_change: bool,
    pub disk_dir: Option<String>,
}

impl From<DiskArgsHelper> for DiskArgs {
    fn from(value: DiskArgsHelper) -> Self {
        Self {
            connection_config: value.connection_config,
            sync_to_disk_on_cache_change: value.sync_to_disk_on_cache_change,
            disk_dir: value.disk_dir,
        }
    }
}

#[derive(FromMeta)]
struct DiskArgsHelper {
    #[darling(default)]
    pub connection_config: Option<String>,
    #[darling(default)]
    pub sync_to_disk_on_cache_change: bool,
    #[darling(default)]
    pub disk_dir: Option<String>,
}

impl FromMeta for DiskArgs {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let helper = DiskArgsHelper::from_list(items)?;
        Ok(helper.into())
    }

    fn from_word() -> darling::Result<Self> {
        Self::from_list(&[])
    }
}

// struct IntOrStr {
// Int(u64),
// Str(String),
// }
// impl FromMeta for IntOrStr {
//     fn from_value(value: &Lit) -> darling::Result<Self> {
//         match value {
//             Lit::Int(n) => {
//                 let n = n.base10_parse::<i64>().unwrap();
//                 if n < 0 {
//                     return Err(darling::Error::custom(
//                         "The complexity must be greater than or equal to 0.",
//                     ));
//                 }
//                 Ok(Self::Int(n as u64))
//             }
//             Lit::Str(s) => Ok(Self::Str(s.value())),
//             _ => Err(darling::Error::unexpected_lit_type(value)),
//         }
//     }
// }

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
        let output = &input.sig.output;
        let output_ty = match output {
            ReturnType::Default => quote! {()},
            ReturnType::Type(_, ty) => quote! {#ty},
        };

        let output_ts = TokenStream::from(output_ty);
        let output_parts = get_output_parts(&output_ts);
        let output_string = output_parts.join("::");
        let output_type_display = output_ts.to_string().replace(' ', "");

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
                                        "#[io_kash] unable to determine a cache value type, found {:?}",
                                        output_type_display
                                    );
                                }
                            } else {
                                panic!(
                                    "#[io_kash] unable to determine a cache value type, found {:?}",
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
