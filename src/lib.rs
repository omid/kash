/*!
[![crates.io version](https://img.shields.io/crates/v/kash.svg?style=flat-square)](https://crates.io/crates/kash)
![build status](https://img.shields.io/github/actions/workflow/status/omid/kash/build.yml?style=flat-square)
[![downloads](https://img.shields.io/crates/d/kash.svg?style=flat-square)](https://crates.io/crates/kash)
[![docs.rs docs](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/kash)
![MIT licensed](https://img.shields.io/crates/l/kash.svg?style=flat-square)
[![dependency status](https://deps.rs/crate/kash/latest/status.svg?style=flat-square)](https://deps.rs/crate/kash)

Function and method cache and memoization library for Rust, using [`#[kash]`](kash) macro.

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
#[kash(size = "100", ttl = "TTL")]
fn fib(n: u64) -> u64 {
    if n == 0 || n == 1 { return n }
    fib(n-1) + fib(n-2)
}
```

## Features

- `default`: Includes `ahash` feature.
- `ahash`: Enable `ahash` hasher as default hashing algorithm.
- `async`: Include support for async functions.
- `redis_store`: Include Redis cache store.
- `redis_tokio`: Include async Redis support using `tokio` and `tokio` tls support, implies `redis_store` and `async`.
- `redis_connection_manager`: Enable the optional `connection-manager` feature of `redis`. Any async redis caches created
  will use a connection manager instead of a `MultiplexedConnection`.
- `redis_ahash`: Enable the optional `ahash` feature of `redis`.
- `disk_store`: Include disk cache store.

----

```rust
use std::thread::sleep;
use std::time::Duration;
use kash::kash;

/// Use an explicit cache-type with a custom creation block and custom cache-key generating block
#[kash(
    size = "100",
    key(ty = "String", expr = r#"{ format!("{}{}", a, b) }"#)
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
use kash::{kash, RedisCacheError};
use kash::AsyncRedisCache;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum ExampleError {
    #[error("error with redis cache `{0}`")]
    RedisError(String),
}

impl From<RedisCacheError> for ExampleError {
    fn from(e: RedisCacheError) -> Self {
        ExampleError::RedisError(format!("{:?}", e))
    }
}

/// Cache the results of an async function in redis. Cache
/// keys will be prefixed with `cache_redis_prefix`.
#[kash(redis)]
async fn async_kash_sleep_secs(secs: u64) -> Result<String, ExampleError> {
    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(secs.to_string())
}
```

----

```rust
use kash::{kash, DiskCacheError};
use kash::DiskCache;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum ExampleError {
    #[error("error with disk cache `{0}`")]
    DiskError(String),
}

impl From<DiskCacheError> for ExampleError {
    fn from(e: DiskCacheError) -> Self {
        ExampleError::DiskError(format!("{:?}", e))
    }
}

/// Cache the results of a function on disk.
/// Cache files will be stored under the system cache dir
/// unless otherwise specified with `dir` or the `create` argument.
#[kash(disk)]
fn kash_sleep_secs(secs: u64) -> Result<String, ExampleError> {
    std::thread::sleep(std::time::Duration::from_secs(secs));
    Ok(secs.to_string())
}
```

Functions defined via macros will have their result, cached using the
function's arguments as a key by default.

When a macro-defined function is called, the function's cache is first checked for an already
computed (and still valid) value before evaluating the function body.

See [`examples`](https://github.com/omid/kash/tree/master/examples) directory for more examples.
*/

#![cfg_attr(docsrs, feature(doc_cfg))]

#[doc(hidden)]
pub use moka;
#[doc(hidden)]
pub use once_cell;

#[cfg(feature = "async")]
use async_trait::async_trait;

#[doc(inline)]
pub use kash_macros::kash;

#[cfg(feature = "redis_tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis_tokio")))]
pub use stores::AsyncRedisCache;
#[cfg(feature = "disk_store")]
#[cfg_attr(docsrs, doc(cfg(feature = "disk_store")))]
pub use stores::{DiskCache, DiskCacheError};
#[cfg(feature = "redis_store")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis_store")))]
pub use stores::{RedisCache, RedisCacheError};

pub mod stores;
#[doc(hidden)]
pub use instant;

#[cfg(feature = "tokio")]
#[doc(hidden)]
pub mod async_sync {
    pub use tokio::sync::Mutex;
    pub use tokio::sync::OnceCell;
    pub use tokio::sync::RwLock;
}

/// Cache operations on an io-connected store
pub trait IOKash<K, V> {
    type Error;

    /// Attempt to retrieve a cached value
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

    /// Remove a cached value
    ///
    /// # Errors
    ///
    /// Should return `Self::Error` if the operation fails
    fn remove(&self, k: &K) -> Result<Option<V>, Self::Error>;

    /// Return the ttl of cached values (time to eviction)
    fn ttl(&self) -> Option<u64> {
        None
    }

    /// Set the ttl of cached values, returns the old value.
    fn set_ttl(&mut self, _seconds: u64) -> Option<u64> {
        None
    }

    /// Remove the ttl for cached values, returns the old value.
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

    /// Remove a cached value
    async fn remove(&self, k: &K) -> Result<Option<V>, Self::Error>;

    /// Return the ttl of cached values (time to eviction)
    fn ttl(&self) -> Option<u64> {
        None
    }

    /// Set the ttl of cached values, returns the old value
    fn set_ttl(&mut self, _seconds: u64) -> Option<u64> {
        None
    }

    /// Remove the ttl for cached values, returns the old value.
    ///
    /// For cache implementations that don't support retaining values indefinitely, this method is
    /// a no-op.
    fn unset_ttl(&mut self) -> Option<u64> {
        None
    }
}
