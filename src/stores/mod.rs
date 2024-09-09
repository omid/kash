#[cfg(feature = "async")]
use std::collections::hash_map::Entry;

#[cfg(feature = "async")]
use {super::KashAsync, async_trait::async_trait, futures::Future};

#[cfg(feature = "disk_store")]
mod disk;
mod memory;
#[cfg(feature = "redis_store")]
mod redis;

#[cfg(feature = "disk_store")]
pub use crate::stores::disk::{DiskCache, DiskCacheBuildError, DiskCacheBuilder, DiskCacheError};
#[cfg(feature = "redis_store")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis_store")))]
pub use crate::stores::redis::{
    RedisCache, RedisCacheBuildError, RedisCacheBuilder, RedisCacheError,
};
pub use memory::MemoryCache;

#[cfg(all(
    feature = "async",
    feature = "redis_store",
    any(feature = "redis_async_std", feature = "redis_tokio")
))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "async",
        feature = "redis_store",
        any(feature = "redis_async_std", feature = "redis_tokio")
    )))
)]
pub use crate::stores::redis::{AsyncRedisCache, AsyncRedisCacheBuilder};
