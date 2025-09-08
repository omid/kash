mod common;
mod io;
mod mem;

use crate::common::macro_args::MacroArgs;
use io::{disk, redis};
use proc_macro::TokenStream;
use syn::{ItemFn, parse_macro_input};

/// Define a memoized function
///
/// By default, it keeps the cache in memory unless you define `disk` or `redis`.
///
/// In the attribute list below, `size`, `eviction_policy` are possible just if it's a memory cache.
///
/// # Attributes
/// - `name`: (optional, string) Specify the name for the generated cache. Defaults to CONSTANT_CASE name of the function
/// - `size`: (optional, string) Specify to keep the number of entries in the cache. Default to unbounded.
/// - `eviction_policy`: (optional, string) Specify the eviction policy, valid options are "lfu" (Least Frequently Used) and "lru" (Least Recently Used). Defaults to "lfu" and it's the most suitable policy for most cases.
/// - `ttl`: (optional, string) Specify a cache TTL in seconds. Defaults to unlimited amount of time.
/// - `key`: (optional, string) Specify a specific key to use. You need to define the following attributes for a custom `key`, e.g., `key(ty = "String", expr = r#"{ format!("{}:{}", arg1, arg2) }"#)`. By default, use all the arguments of the function as the key.
///   - `ty`: (string) Specify type of the key. E.g, `ty = "String"`
///   - `expr`: (string expr) Specify an expression used to generate a cache key.
///     E.g., `expr = r#"{ format!("{}:{}", arg1, arg2) }"#`.
/// - `result`: (optional) If your function returns a `Result`, only cache `Ok` values returned by the function.
/// - `option`: (optional) If your function returns an `Option`, only cache `Some` values returned by the function.
/// - `in_impl`: (optional) Set it if your function is defined in an `impl` block, otherwise not.
/// - `redis`: (optional) Store cached values in Redis.
///   - `prefix_block`: (optional, string expr) specify an expression used to create the string used as a
///     prefix for all cache keys of this function, e.g. `prefix_block = r#"{ "my_prefix:" }"#`.
///     When not specified, the cache prefix will be constructed from the name of the function. This
///     could result in unexpected conflicts between kash-functions of the same name, be sure to specify a
///     `prefix_block` if you have multiple kash-functions with the same name. And consider using a unique
///     separator at the end of the prefix, like ":" in the example above.
/// - `disk`: (optional) Store cached values on disk.
///   - `dir`: (optional, string) Specify directory of `disk` cache
///   - `sync_to_disk_on_cache_change`: (optional) Specify whether to synchronize the cache to disk each
///     time the cache changes.
///   - `connection_config`: (optional, string expr) Specify an expression which returns a `sled::Config`
///     to give more control over the connection to the `disk` cache, i.e., useful for controlling the rate at which the cache syncs to disk.
///     See the docs of `kash::stores::DiskCacheBuilder::connection_config` for more info.
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
    };

    if args.redis.is_some() {
        redis::kash(&input, &args)
    } else if args.disk.is_some() {
        disk::kash(&input, &args)
    } else {
        mem::kash(&input, &args)
    }
}
