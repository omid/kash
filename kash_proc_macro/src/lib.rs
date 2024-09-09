mod helpers;
mod io_kash;
mod kash;

use proc_macro::TokenStream;

/// Define a memoized function using a cache store that implements `kash::Kash` (and
/// `kash::KashAsync` for async functions)
///
/// # Attributes
/// - `name`: (optional, string) specify the name for the generated cache, defaults to the function name uppercase.
/// - `size`: (optional, usize) specify an LRU max size, implies the cache type is a `SizedCache` or `TimedSizedCache`.
/// - `time`: (optional, u64) specify a cache TTL in seconds, implies the cache type is a `TimedCache` or `TimedSizedCache`.
/// - `time_refresh`: (optional, bool) specify whether to refresh the TTL on cache hits.
/// - `sync_writes`: (optional, bool) specify whether to synchronize the execution of writing of unkash values.
/// - `ty`: (optional, string type) The cache store type to use. Defaults to `UnboundCache`. When `unbound` is
///   specified, defaults to `UnboundCache`. When `size` is specified, defaults to `SizedCache`.
///   When `time` is specified, defaults to `TimedKash`.
///   When `size` and `time` are specified, defaults to `TimedSizedCache`. When `ty` is
///   specified, `create` must also be specified.
/// - `create`: (optional, string expr) specify an expression used to create a new cache store, e.g. `create = r##"{ CacheType::new() }"##`.
/// - `key`: (optional, string type) specify what type to use for the cache key, e.g. `key = "u32"`.
///    When `key` is specified, `convert` must also be specified.
/// - `convert`: (optional, string expr) specify an expression used to convert function arguments to a cache
///   key, e.g. `convert = r##"{ format!("{}:{}", arg1, arg2) }"##`. When `convert` is specified,
///   `key` or `ty` must also be set.
/// - `result`: (optional, bool) If your function returns a `Result`, only cache `Ok` values returned by the function.
/// - `option`: (optional, bool) If your function returns an `Option`, only cache `Some` values returned by the function.
/// - `wrap_return`: (optional, bool) If your function returns a `kash::Return` or `Result<kash::Return, E>`,
///   the `kash::Return.was_cached` flag will be updated when a kash value is returned.
/// - `result_fallback`: (optional, bool) If your function returns a `Result` and it fails, the cache will instead refresh the recently expired `Ok` value.
///   In other words, refreshes are best-effort - returning `Ok` refreshes as usual but `Err` falls back to the last `Ok`.
///   This is useful, for example, for keeping the last successful result of a network operation even during network disconnects.
/// - `in_impl`: (optional, bool) If your function is defined in an `impl` block, set this to `true`.
///
/// ## Note
/// The `ty`, `create`, `key`, and `convert` attributes must be in a `String`
/// This is because darling, which is used for parsing the attributes, does not support directly parsing
/// attributes into `Type`s or `Block`s.
#[proc_macro_attribute]
pub fn kash(args: TokenStream, input: TokenStream) -> TokenStream {
    kash::kash(args, input)
}

/// Define a memoized function using a cache store that implements `kash::IOKash` (and
/// `kash::IOKashAsync` for async functions)
///
/// # Attributes
/// - `map_error`: (string, expr closure) specify a closure used to map any IO-store errors into
///   the error type returned by your function.
/// - `name`: (optional, string) specify the name for the generated cache, defaults to the function name uppercase.
/// - `redis`: (optional, bool, default: false) default to a `RedisCache` or `AsyncRedisCache`
/// - `disk`: (optional, bool, default: false) use a `DiskCache`, this must be set to true even if `type` and `create` are specified.
/// - `time`: (optional, u64) specify a cache TTL in seconds, implies the cache type is a `TimedKash` or `TimedSizedCache`.
/// - `time_refresh`: (optional, bool) specify whether to refresh the TTL on cache hits.
/// - `ty`: (optional, string type) explicitly specify the cache store type to use.
/// - `cache_prefix_block`: (optional, string expr) specify an expression used to create the string used as a
///   prefix for all cache keys of this function, e.g. `cache_prefix_block = r##"{ "my_prefix" }"##`.
///   When not specified, the cache prefix will be constructed from the name of the function. This
///   could result in unexpected conflicts between io_kash-functions of the same name, so it's
///   recommended that you specify a prefix you're sure will be unique.
/// - `create`: (optional, string expr) specify an expression used to create a new cache store, e.g. `create = r##"{ CacheType::new() }"##`.
/// - `key`: (optional, string type) specify what type to use for the cache key, e.g. `ty = "TimedKash<u32, u32>"`.
///    When `key` is specified, `convert` must also be specified.
/// - `convert`: (optional, string expr) specify an expression used to convert function arguments to a cache
///   key, e.g. `convert = r##"{ format!("{}:{}", arg1, arg2) }"##`. When `convert` is specified,
///   `key` or `ty` must also be set.
/// - `wrap_return`: (optional, bool, default: false) If your function returns a `kash::Return` or `Result<kash::Return, E>`,
///   the `kash::Return.was_cached` flag will be updated when a kash value is returned.
/// - `sync_to_disk_on_cache_change`: (optional, bool) in the case of `DiskCache` specify whether to synchronize the cache to disk each
///   time the cache changes.
/// - connection_config: (optional, string expr) specify an expression which returns a `sled::Config`
///   to give more control over the connection to the disk cache, i.e. useful for controlling the rate at which the cache syncs to disk.
///   See the docs of `kash::stores::DiskCacheBuilder::connection_config` for more info.
/// - `in_impl`: (optional, bool, default: false) If your function is defined in an `impl` block, set this to `true`.
///
/// ## Note
/// The `ty`, `create`, `key`, and `convert` attributes must be in a `String`
/// This is because darling, which is used for parsing the attributes, does not support directly parsing
/// attributes into `Type`s or `Block`s.
#[proc_macro_attribute]
pub fn io_kash(args: TokenStream, input: TokenStream) -> TokenStream {
    io_kash::io_kash(args, input)
}
