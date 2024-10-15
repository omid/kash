use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use std::ops::Deref;
use quote::quote;
use syn::{Error, ItemFn, PathArguments, ReturnType, Type};

#[derive(FromMeta, Clone, Debug)]
#[darling(and_then = "Self::init_validate")]
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

    #[darling(default)]
    pub size: Option<String>,
    #[darling(default)]
    pub sync_writes: bool,

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

    pub fn init_validate(self) -> darling::Result<Self> {
        let mut acc = darling::Error::accumulator();
        if self.disk.is_some() && self.redis.is_some() {
            acc.push(darling::Error::custom(
                "`disk` and `redis` are mutually exclusive",
            ));
        }
        
        if self.result && self.option {
            acc.push(darling::Error::custom(
                "the `result` and `option` attributes are mutually exclusive"
            ));
        }
        acc.finish_with(self)
    }

    pub fn validate(&self, input: &ItemFn) -> darling::Result<()> {
        let output = &input.sig.output;
        
        let mut acc = darling::Error::accumulator();

        match output {
            ReturnType::Default => {
                let output_ty = match output {
                    ReturnType::Default => quote! {()},
                    ReturnType::Type(_, ty) => quote! {#ty},
                };
                acc.push(darling::Error::custom(format!(
                    "`disk` and `redis` caches must return `Result`, found {:?}",
                    output_ty.to_string().replace(' ', "")
                )));
            }
            ReturnType::Type(_, ty) => {
                if let Type::Path(typepath) = ty.deref() {
                    let segments = &typepath.path.segments;
                    if let PathArguments::AngleBracketed(_) =
                        &segments.last().unwrap().arguments
                    {
                    } else {
                        acc.push(darling::Error::custom(
                            "`disk` and `redis` caches must return `Result`",
                        ));
                    }
                } else {
                    acc.push(
                        darling::Error::custom(
                            "function return type too complex. `disk` and `redis` caches must return `Result`",
                        )
                    );
                }
            }
        };

        acc.finish_with(())
    }
}
