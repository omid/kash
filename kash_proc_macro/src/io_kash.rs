use crate::{functions::io::macro_args::MacroArgs, helpers::*};
use darling::{ast::NestedMeta, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_str, Block, Expr, ExprClosure, GenericArgument, Ident, ItemFn,
    PathArguments, ReturnType, Type,
};

pub fn io_kash(args: TokenStream, input: TokenStream) -> TokenStream {
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

    if let Some(error) = args.validate(&input) {
        return error;
    }

    // pull out the parts of the input
    let mut attributes = input.attrs;
    let visibility = input.vis;
    let signature = input.sig;
    let body = input.block;

    // pull out the parts of the function signature
    let fn_ident = signature.ident.clone();
    let inputs = signature.inputs.clone();
    let asyncness = signature.asyncness;
    let generics = signature.generics.clone();

    let (_, without_self_types) = get_input_types(&inputs);
    let (_, without_self_names) = get_input_names(&inputs);

    let output = &signature.output;
    let output_ty = match output {
        ReturnType::Default => quote! {()},
        ReturnType::Type(_, ty) => quote! {#ty},
    };

    let output_ts = TokenStream::from(output_ty);
    let output_parts = get_output_parts(&output_ts);
    let output_string = output_parts.join("::");

    // Find the type of the value to store.
    // Return type always needs to be a result, so we want the (first) inner type.
    // For Result<i32, String>, store i32, etc.
    let cache_value_ty = match output {
        ReturnType::Type(_, ty) => match **ty {
            Type::Path(ref typepath) => {
                let segments = &typepath.path.segments;
                if let PathArguments::AngleBracketed(ref brackets) =
                    segments.last().unwrap().arguments
                {
                    let inner_ty = brackets.args.first().unwrap();
                    if output_string.contains("Return") || output_string.contains("kash::Return") {
                        if let GenericArgument::Type(Type::Path(ref typepath)) = inner_ty {
                            let segments = &typepath.path.segments;
                            if let PathArguments::AngleBracketed(ref brackets) =
                                segments.last().unwrap().arguments
                            {
                                let inner_ty = brackets.args.first().unwrap();
                                quote! {#inner_ty}
                            } else {
                                quote! {}
                            }
                        } else {
                            quote! {}
                        }
                    } else {
                        quote! {#inner_ty}
                    }
                } else {
                    quote! {}
                }
            }
            _ => quote! {},
        },
        _ => unreachable!("error earlier caught"),
    };

    // make the cache identifier
    let cache_ident = match args.name {
        Some(ref name) => Ident::new(name, fn_ident.span()),
        None => Ident::new(&fn_ident.to_string().to_uppercase(), fn_ident.span()),
    };
    let cache_name = cache_ident.to_string();

    let (cache_key_ty, key_convert_block) = make_cache_key_type(
        &args.convert,
        &args.ty,
        without_self_types,
        &without_self_names,
    );

    // make the cache type and create statement
    let (cache_ty, cache_create) = match (
        &args.redis,
        &args.disk,
        &args.time,
        &args.time_refresh,
        &args.cache_prefix_block,
        &args.ty,
        &args.create,
        &args.sync_to_disk_on_cache_change,
        &args.connection_config,
    ) {
        // redis
        (true, false, time, time_refresh, cache_prefix, ty, cache_create, _, _) => {
            let cache_ty = match ty {
                Some(ty) => {
                    let ty = parse_str::<Type>(ty).expect("unable to parse cache type");
                    quote! { #ty }
                }
                None => {
                    if asyncness.is_some() {
                        quote! { kash::AsyncRedisCache<#cache_key_ty, #cache_value_ty> }
                    } else {
                        quote! { kash::RedisCache<#cache_key_ty, #cache_value_ty> }
                    }
                }
            };
            let cache_create = match cache_create {
                Some(cache_create) => {
                    if time.is_some() || time_refresh.is_some() || cache_prefix.is_some() {
                        panic!("cannot specify `time`, `time_refresh`, or `cache_prefix` when passing `create block");
                    } else {
                        let cache_create = parse_str::<Block>(cache_create.as_ref())
                            .expect("unable to parse cache create block");
                        quote! { #cache_create }
                    }
                }
                None => {
                    if time.is_none() {
                        if asyncness.is_some() {
                            panic!("AsyncRedisCache requires a `time` when `create` block is not specified")
                        } else {
                            panic!(
                                "RedisCache requires a `time` when `create` block is not specified"
                            )
                        };
                    } else {
                        let cache_prefix = if let Some(cp) = cache_prefix {
                            cp.to_string()
                        } else {
                            format!(" {{ \"kash::proc_macro::io_kash::{}\" }}", cache_ident)
                        };
                        let cache_prefix = parse_str::<Block>(cache_prefix.as_ref())
                            .expect("unable to parse cache_prefix_block");
                        match time_refresh {
                            Some(time_refresh) => {
                                if asyncness.is_some() {
                                    quote! { kash::AsyncRedisCache::new(#cache_prefix, #time).set_refresh(#time_refresh).build().await.expect("error constructing AsyncRedisCache in #[io_kash] macro") }
                                } else {
                                    quote! {
                                        kash::RedisCache::new(#cache_prefix, #time).set_refresh(#time_refresh).build().expect("error constructing RedisCache in #[io_kash] macro")
                                    }
                                }
                            }
                            None => {
                                if asyncness.is_some() {
                                    quote! { kash::AsyncRedisCache::new(#cache_prefix, #time).build().await.expect("error constructing AsyncRedisCache in #[io_kash] macro") }
                                } else {
                                    quote! {
                                        kash::RedisCache::new(#cache_prefix, #time).build().expect("error constructing RedisCache in #[io_kash] macro")
                                    }
                                }
                            }
                        }
                    }
                }
            };
            (cache_ty, cache_create)
        }
        // disk
        (
            false,
            true,
            time,
            time_refresh,
            _,
            ty,
            cache_create,
            sync_to_disk_on_cache_change,
            connection_config,
        ) => {
            let cache_ty = match ty {
                Some(ty) => {
                    let ty = parse_str::<Type>(ty).expect("unable to parse cache type");
                    quote! { #ty }
                }
                None => {
                    // https://github.com/spacejam/sled?tab=readme-ov-file#interaction-with-async
                    quote! { kash::DiskCache<#cache_key_ty, #cache_value_ty> }
                }
            };
            let connection_config = match connection_config {
                Some(connection_config) => {
                    let connection_config = parse_str::<Expr>(connection_config)
                        .expect("unable to parse connection_config block");
                    Some(quote! { #connection_config })
                }
                None => None,
            };
            let cache_create = match cache_create {
                Some(cache_create) => {
                    if time.is_some() || time_refresh.is_some() {
                        panic!(
                            "cannot specify `time` or `time_refresh` when passing `create block"
                        );
                    } else {
                        let cache_create = parse_str::<Block>(cache_create.as_ref())
                            .expect("unable to parse cache create block");
                        quote! { #cache_create }
                    }
                }
                None => {
                    let mut create = quote! {
                        kash::DiskCache::new(#cache_name)
                    };
                    if let Some(time) = time {
                        create = quote! {
                            (#create).set_ttl(#time)
                        };
                    };
                    if let Some(time_refresh) = time_refresh {
                        create = quote! {
                            (#create).set_refresh(#time_refresh)
                        };
                    }
                    if let Some(sync_to_disk_on_cache_change) = sync_to_disk_on_cache_change {
                        create = quote! {
                            (#create).set_sync_to_disk_on_cache_change(#sync_to_disk_on_cache_change)
                        };
                    };
                    if let Some(connection_config) = connection_config {
                        create = quote! {
                            (#create).set_connection_config(#connection_config)
                        };
                    };
                    if let Some(disk_dir) = args.disk_dir {
                        create = quote! { (#create).set_disk_directory(#disk_dir) };
                    }
                    quote! { (#create).build().expect("error constructing DiskCache in #[io_kash] macro") }
                }
            };
            (cache_ty, cache_create)
        }
        (_, _, time, time_refresh, cache_prefix, ty, cache_create, _, _) => {
            let cache_ty = match ty {
                Some(ty) => {
                    let ty = parse_str::<Type>(ty).expect("unable to parse cache type");
                    quote! { #ty }
                }
                None => panic!("#[io_kash] cache `ty` must be specified"),
            };
            let cache_create = match cache_create {
                Some(cache_create) => {
                    if time.is_some() || time_refresh.is_some() || cache_prefix.is_some() {
                        panic!("cannot specify `time`, `time_refresh`, or `cache_prefix` when passing `create block");
                    } else {
                        let cache_create = parse_str::<Block>(cache_create.as_ref())
                            .expect("unable to parse cache create block");
                        quote! { #cache_create }
                    }
                }
                None => {
                    panic!("#[io_kash] cache `create` block must be specified");
                }
            };
            (cache_ty, cache_create)
        }
        #[allow(unreachable_patterns)]
        _ => panic!("#[io_kash] cache types cache type could not be determined"),
    };

    let map_error = &args.map_error;
    let map_error = parse_str::<ExprClosure>(map_error).expect("unable to parse map_error block");

    // make the set cache and return cache blocks
    let (set_cache_block, return_cache_block) = {
        let (set_cache_block, return_cache_block) = if args.wrap_return {
            (
                if asyncness.is_some() && !args.disk {
                    quote! {
                        if let Ok(result) = &result {
                            cache.set(key, result.value.clone()).await.map_err(#map_error)?;
                        }
                    }
                } else {
                    quote! {
                        if let Ok(result) = &result {
                            cache.set(key, result.value.clone()).map_err(#map_error)?;
                        }
                    }
                },
                quote! { let mut r = ::kash::Return::new(result.clone()); r.was_cached = true; return Ok(r) },
            )
        } else {
            (
                if asyncness.is_some() && !args.disk {
                    quote! {
                        if let Ok(result) = &result {
                            cache.set(key, result.clone()).await.map_err(#map_error)?;
                        }
                    }
                } else {
                    quote! {
                        if let Ok(result) = &result {
                            cache.set(key, result.clone()).map_err(#map_error)?;
                        }
                    }
                },
                quote! { return Ok(result.clone()) },
            )
        };
        (set_cache_block, return_cache_block)
    };

    let set_cache_and_return = quote! {
        #set_cache_block
        result
    };

    let signature_no_muts = get_mut_signature(signature.clone());

    // create a signature for the cache-priming function
    let prime_fn_ident = Ident::new(&format!("{}_prime_cache", &fn_ident), fn_ident.span());
    let mut prime_sig = signature_no_muts.clone();
    prime_sig.ident = prime_fn_ident;

    // make kash static, kash function and prime kash function doc comments
    let cache_ident_doc = format!("Kash static for the [`{}`] function.", fn_ident);
    let no_cache_fn_indent_doc = format!("Origin of the kash function [`{}`].", fn_ident);
    let prime_fn_indent_doc = format!("Primes the kash function [`{}`].", fn_ident);
    let cache_fn_doc_extra = format!(
        "This is a kash function that uses the [`{}`] kash static.",
        cache_ident
    );
    fill_in_attributes(&mut attributes, cache_fn_doc_extra);

    let async_cache_get_return = if asyncness.is_some() && !args.disk {
        quote! {
            if let Some(result) = cache.get(&key).await.map_err(#map_error)? {
                #return_cache_block
            }
        }
    } else {
        quote! {
            if let Some(result) = cache.get(&key).map_err(#map_error)? {
                #return_cache_block
            }
        }
    };

    let use_trait = if asyncness.is_some() && !args.disk {
        quote! { use kash::IOKashAsync; }
    } else {
        quote! { use kash::IOKash; }
    };

    let no_cache_fn_ident = Ident::new(&format!("{}_no_cache", &fn_ident), fn_ident.span());
    let fn_cache_ident = Ident::new(&format!("{}_get_cache_ident", &fn_ident), fn_ident.span());

    let call_prefix = if args.in_impl {
        quote! { Self:: }
    } else {
        quote! {}
    };

    let init_cache_ident = if args.in_impl {
        quote! {
            &#call_prefix #fn_cache_ident()
        }
    } else {
        quote! {
            &#call_prefix #cache_ident
        }
    };

    let init;
    let function_no_cache;
    let function_call;
    let ty;
    let logic;
    if asyncness.is_some() {
        init = quote! {
            let init = || async { #cache_create };
        };

        function_no_cache = quote! {
            async fn #no_cache_fn_ident #generics (#inputs) #output #body
        };

        function_call = quote! {
            let result = #call_prefix #no_cache_fn_ident(#(#without_self_names),*).await;
        };

        if args.in_impl {
            ty = quote! {
                #visibility fn #fn_cache_ident() -> &'static ::kash::async_sync::OnceCell<#cache_ty> {
                    static #cache_ident: ::kash::async_sync::OnceCell<#cache_ty> = ::kash::async_sync::OnceCell::const_new();
                    &#cache_ident
                }
            };
        } else {
            ty = quote! {
                #visibility static #cache_ident: ::kash::async_sync::OnceCell<#cache_ty> = ::kash::async_sync::OnceCell::const_new();
            };
        }
        logic = quote! {
            let cache = #init_cache_ident.get_or_init(init).await;
            #async_cache_get_return
        };
    } else {
        init = quote! {};

        function_no_cache = quote! {
            fn #no_cache_fn_ident #generics (#inputs) #output #body
        };

        function_call = quote! {
            let result = #call_prefix #no_cache_fn_ident(#(#without_self_names),*);
        };

        if args.in_impl {
            ty = quote! {
                #visibility fn #fn_cache_ident() -> &'static ::kash::once_cell::sync::Lazy<#cache_ty> {
                    static #cache_ident: ::kash::once_cell::sync::Lazy<#cache_ty> = ::kash::once_cell::sync::Lazy::new(|| #cache_create);
                    &#cache_ident
                }
            };
        } else {
            ty = quote! {
                #visibility static #cache_ident: ::kash::once_cell::sync::Lazy<#cache_ty> = ::kash::once_cell::sync::Lazy::new(|| #cache_create);
            };
        }
        logic = quote! {
            let cache = #init_cache_ident;
            if let Some(result) = cache.get(&key).map_err(#map_error)? {
                #return_cache_block
            }
        };
    }

    let do_set_return_block = if asyncness.is_some() {
        quote! {
            #function_call
            let cache = #init_cache_ident.get_or_init(init).await;
            #set_cache_and_return
        }
    } else {
        quote! {
            #function_call
            let cache = #init_cache_ident;
            #set_cache_and_return
        }
    };

    // put it all together
    let expanded = quote! {
        // Kash static
        #[doc = #cache_ident_doc]
        #ty
        // No cache function (origin of the kash function)
        #[doc = #no_cache_fn_indent_doc]
        #(#attributes)*
        #visibility #function_no_cache
        // Kash function
        #(#attributes)*
        #visibility #signature_no_muts {
            #init
            #use_trait
            let key = #key_convert_block;
            {
                // check if the result is kash
                #logic
            }
            #do_set_return_block
        }
        // Prime kash function
        #[doc = #prime_fn_indent_doc]
        #[allow(dead_code)]
        #(#attributes)*
        #visibility #prime_sig {
            #use_trait
            #init
            let key = #key_convert_block;
            #do_set_return_block
        }
    };

    expanded.into()
}
