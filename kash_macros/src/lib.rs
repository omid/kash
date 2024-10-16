mod common;
mod io;
mod mem;

use crate::common::macro_args::MacroArgs;
use io::{disk, redis};
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn};

/// Define a memoized function
///
/// By default, it keeps the cache in memory, unless you define `disk` or `redis`.
///
/// In the attribute list below, `size` and `sync_writes` are possible just if it's a memory cache.
///
/// # Attributes
/// - `name`: (optional, string) Specify the name for the generated cache, defaults to the function name uppercase.
/// - `size`: (optional, string) Specify to keep the amount of entries in the cache.
/// - `ttl`: (optional, string) Specify a cache TTL in seconds.
/// - `sync_writes`: (optional) Specify whether to synchronize the execution of writing uncached values.
/// - `key`: (optional, string) Specify what type to use for the cache key, e.g. `key = "u32"`.
///    When `key` is specified, `convert` must also be specified.
/// - `convert`: (optional, string expr) Specify an expression used to convert function arguments to a cache
///   key, e.g. `convert = r#"{ format!("{}:{}", arg1, arg2) }"#`. When `convert` is specified, `key` must also be set.
/// - `result`: (optional) If your function returns a `Result`, only cache `Ok` values returned by the function.
/// - `option`: (optional) If your function returns an `Option`, only cache `Some` values returned by the function.
/// - `in_impl`: (optional) Set it if your function is defined in an `impl` block or not.
/// - `redis`: (optional) Store cached values in Redis.
///   - `prefix_block`: (optional, string expr) specify an expression used to create the string used as a
///     prefix for all cache keys of this function, e.g. `prefix_block = r#"{ "my_prefix" }"#`.
///     When not specified, the cache prefix will be constructed from the name of the function. This
///     could result in unexpected conflicts between kash-functions of the same name, so it's
///     recommended that you specify a prefix you're sure will be unique.
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
    }

    if args.redis.is_some() {
        redis::kash(&input, &args)
    } else if args.disk.is_some() {
        disk::kash(&input, &args)
    } else {
        mem::kash(&input, &args)
    }
}
