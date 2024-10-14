/*!
[![Build Status](https://github.com/omid/kash/actions/workflows/build.yml/badge.svg)](https://github.com/omid/kash/actions/workflows/build.yml)
[![crates.io](https://img.shields.io/crates/v/kash.svg)](https://crates.io/crates/kash)
[![docs](https://docs.rs/kash/badge.svg)](https://docs.rs/kash)

Caching structures and simplified function memoization, using [`#[kash]`](kash)/[`#[io_kash]`](io_kash) macros.

```rust
use kash::kash;

/// Defines a function named `fib` that uses a cache implicitly named `FIB`.
/// By default, the cache will be the function's name in all caps.
#[kash]
fn fib(n: u64) -> u64 {
    if n == 0 || n == 1 { return n }
    fib(n-1) + fib(n-2)
}
```

Or if you want to limit the size and time-to-live:

```rust
use kash::kash;

const TTL: u64 = 1000;
#[kash::kash(size = "100", ttl = "TTL")]
fn fib(n: u64) -> u64 {
    if n == 0 || n == 1 { return n }
    fib(n-1) + fib(n-2)
}
```

## Features

- `default`: Includes `ahash` features
- `ahash`: Enable `ahash` hasher as default hashing algorithm.
- `async`: Include support for async functions
- `redis_store`: Include Redis cache store
- `redis_async_std`: Include async Redis support using `async-std` and `async-std` tls support, implies `redis_store` and `async`
- `redis_tokio`: Include async Redis support using `tokio` and `tokio` tls support, implies `redis_store` and `async`
- `redis_connection_manager`: Enable the optional `connection-manager` feature of `redis`. Any async redis caches created
                              will use a connection manager instead of a `MultiplexedConnection`
- `redis_ahash`: Enable the optional `ahash` feature of `redis`
- `disk_store`: Include disk cache store

----

```rust
use std::thread::sleep;
use std::time::Duration;
use kash::kash;

/// Use an explicit cache-type with a custom creation block and custom cache-key generating block
#[kash(
    key = "String",
    size = "100",
    convert = r#"{ format!("{}{}", a, b) }"#
)]
fn keyed(a: &str, b: &str) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}
# pub fn main() { }
```

----

```rust
use kash::io_kash;
use kash::AsyncRedisCache;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum ExampleError {
    #[error("error with redis cache `{0}`")]
    RedisError(String),
}

/// Cache the results of an async function in redis. Cache
/// keys will be prefixed with `cache_redis_prefix`.
/// A `map_error` closure must be specified to convert any
/// redis cache errors into the same type of error returned
/// by your function. All `io_kash` functions must return `Result`s.
#[io_kash(
    map_error = r##"|e| ExampleError::RedisError(format!("{:?}", e))"##,
    redis,
)]
async fn async_kash_sleep_secs(secs: u64) -> Result<String, ExampleError> {
    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(secs.to_string())
}
```

----

```rust
use kash::io_kash;
use kash::DiskCache;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum ExampleError {
    #[error("error with disk cache `{0}`")]
    DiskError(String),
}

/// Cache the results of a function on disk.
/// Cache files will be stored under the system cache dir
/// unless otherwise specified with `disk_dir` or the `create` argument.
/// A `map_error` closure must be specified to convert any
/// disk cache errors into the same type of error returned
/// by your function. All `io_kash` functions must return `Result`s.
#[io_kash(
    map_error = r##"|e| ExampleError::DiskError(format!("{:?}", e))"##,
    disk
)]
fn kash_sleep_secs(secs: u64) -> Result<String, ExampleError> {
    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(secs.to_string())
}
```

Functions defined via macros will have their result, cached using the
function's arguments as a key, a `convert` expression specified.

When a macro-defined function is called, the function's cache is first checked for an already
computed (and still valid) value before evaluating the function body.

See the [`proc_macro`](proc_macro) module, and the
[`examples`](https://github.com/omid/kash/tree/master/examples) directory for more examples.
*/

#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(hidden)]
pub extern crate moka;
#[doc(hidden)]
pub extern crate once_cell;

#[cfg(feature = "async")]
use async_trait::async_trait;
#[cfg_attr(docsrs, doc(cfg(feature = "proc_macro")))]
pub use proc_macro::Return;

#[doc(inline)]
pub use kash_macros::{io_kash, kash};

#[cfg(any(feature = "redis_async_std", feature = "redis_tokio"))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "redis_async_std", feature = "redis_tokio")))
)]
pub use stores::AsyncRedisCache;
#[cfg(feature = "disk_store")]
#[cfg_attr(docsrs, doc(cfg(feature = "disk_store")))]
pub use stores::{DiskCache, DiskCacheError};
#[cfg(feature = "redis_store")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis_store")))]
pub use stores::{RedisCache, RedisCacheError};

pub mod proc_macro;
pub mod stores;
#[doc(hidden)]
pub use web_time;

#[cfg(feature = "async")]
#[doc(hidden)]
pub mod async_sync {
    pub use tokio::sync::Mutex;
    pub use tokio::sync::OnceCell;
    pub use tokio::sync::RwLock;
}

/// Cache operations on an io-connected store
pub trait IOKash<K, V> {
    type Error;

    /// Attempt to retrieve a kash value
    ///
    /// # Errors
    ///
    /// Should return `Self::Error` if the operation fails
    fn get(&self, k: &K) -> Result<Option<V>, Self::Error>;

    /// Insert a key, value pair and return the previous value
    ///
    /// # Errors
    ///
    /// Should return `Self::Error` if the operation fails
    fn set(&self, k: K, v: V) -> Result<Option<V>, Self::Error>;

    /// Remove a kash value
    ///
    /// # Errors
    ///
    /// Should return `Self::Error` if the operation fails
    fn remove(&self, k: &K) -> Result<Option<V>, Self::Error>;

    /// Return the ttl of kash values (time to eviction)
    fn ttl(&self) -> Option<u64> {
        None
    }

    /// Set the ttl of kash values, returns the old value.
    fn set_ttl(&mut self, _seconds: u64) -> Option<u64> {
        None
    }

    /// Remove the ttl for kash values, returns the old value.
    ///
    /// For cache implementations that don't support retaining values indefinitely, this method is
    /// a no-op.
    fn unset_ttl(&mut self) -> Option<u64> {
        None
    }
}

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[async_trait]
pub trait IOKashAsync<K, V> {
    type Error;
    async fn get(&self, k: &K) -> Result<Option<V>, Self::Error>;

    async fn set(&self, k: K, v: V) -> Result<Option<V>, Self::Error>;

    /// Remove a kash value
    async fn remove(&self, k: &K) -> Result<Option<V>, Self::Error>;

    /// Return the ttl of kash values (time to eviction)
    fn ttl(&self) -> Option<u64> {
        None
    }

    /// Set the ttl of kash values, returns the old value
    fn set_ttl(&mut self, _seconds: u64) -> Option<u64> {
        None
    }

    /// Remove the ttl for kash values, returns the old value.
    ///
    /// For cache implementations that don't support retaining values indefinitely, this method is
    /// a no-op.
    fn unset_ttl(&mut self) -> Option<u64> {
        None
    }
}
