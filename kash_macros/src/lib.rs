mod common;
mod mem;
mod io;

use crate::common::macro_args::MacroArgs;
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};
use io::{disk, redis};

/// Define a memoized function
///
/// # Attributes
/// - `name`: (optional, string) specify the name for the generated cache, defaults to the function name uppercase.
/// - `size`: (optional, string) specify to keep the amount of entries in the cache.
/// - `ttl`: (optional, string) specify a cache TTL in seconds.
/// - `sync_writes`: (optional) specify whether to synchronize the execution of writing uncached values.
/// - `key`: (optional, string) specify what type to use for the cache key, e.g. `key = "u32"`.
///    When `key` is specified, `convert` must also be specified.
/// - `convert`: (optional, string expr) specify an expression used to convert function arguments to a cache
///   key, e.g. `convert = r#"{ format!("{}:{}", arg1, arg2) }"#`. When `convert` is specified, `key` must also be set.
/// - `result`: (optional) If your function returns a `Result`, only cache `Ok` values returned by the function.
/// - `option`: (optional) If your function returns an `Option`, only cache `Some` values returned by the function.
/// - `in_impl`: (optional) If your function is defined in an `impl` block or not.
///
#[proc_macro_attribute]
pub fn kash(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = match MacroArgs::try_from(args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };

    let input = parse_macro_input!(input as ItemFn);

    match args.validate(&input).map_err(|e| e.write_errors()) {
        Ok(_) => {}
        Err(e) => return e.into(),
    }

    if args.redis.is_some() {
        redis::kash(&input, &args)
    } else if args.disk.is_some() {
        disk::kash(&input, &args)
    } else {
        mem::kash(&input, &args)
    }
}

// /// Define a memoized function that implements `kash::IOKash` (and `kash::IOKashAsync` for async functions)
// ///
// /// # Attributes
// /// - `name`: (optional, string) specify the name for the generated cache, defaults to the function name uppercase.
// /// - `redis`: (optional) default to a `RedisCache` or `AsyncRedisCache`
// /// - `disk`: (optional) use a `DiskCache`, this must be set to true even if `type` and `create` are specified.
// /// - `ttl`: (optional, string) specify a cache TTL in seconds.
// /// - `cache_prefix_block`: (optional, string expr) specify an expression used to create the string used as a
// ///   prefix for all cache keys of this function, e.g. `cache_prefix_block = r#"{ "my_prefix" }"#`.
// ///   When not specified, the cache prefix will be constructed from the name of the function. This
// ///   could result in unexpected conflicts between kash-functions of the same name, so it's
// ///   recommended that you specify a prefix you're sure will be unique.
// /// - `key`: (optional, string) specify what type to use for the cache key, e.g. `key = "u32"`.
// ///    When `key` is specified, `convert` must also be specified.
// /// - `convert`: (optional, string expr) specify an expression used to convert function arguments to a cache
// ///   key, e.g. `convert = r#"{ format!("{}:{}", arg1, arg2) }"#`. When `convert` is specified,
// ///   `key` or `ty` must also be set.
// /// - `sync_to_disk_on_cache_change`: (optional, bool) in the case of `DiskCache` specify whether to synchronize the cache to disk each
// ///   time the cache changes.
// /// - connection_config: (optional, string expr) specify an expression which returns a `sled::Config`
// ///   to give more control over the connection to the disk cache, i.e., useful for controlling the rate at which the cache syncs to disk.
// ///   See the docs of `kash::stores::DiskCacheBuilder::connection_config` for more info.
// /// - `in_impl`: (optional) If your function is defined in an `impl` block or not.
// ///
// #[proc_macro_attribute]
// pub fn kash(args: TokenStream, input: TokenStream) -> TokenStream {
//     disk::kash(args, input)
// }
