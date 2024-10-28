#![cfg(feature = "redis_store")]

use kash::{kash, RedisCacheError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum TestError {
    #[error("error with redis cache `{0}`")]
    RedisError(String),
    #[error("count `{0}`")]
    Count(u32),
}

impl From<RedisCacheError> for TestError {
    fn from(e: RedisCacheError) -> Self {
        TestError::RedisError(format!("{e:?}"))
    }
}

#[kash(
    redis(prefix_block = "{ \"__kash_redis_proc_macro_test_fn_kash_redis:\" }"),
    ttl = "1"
)]
fn kash_redis(n: u32) -> Result<u32, TestError> {
    if n < 5 {
        Ok(n)
    } else {
        Err(TestError::Count(n))
    }
}

#[test]
fn test_kash_redis() {
    assert_eq!(kash_redis(1), Ok(1));
    assert_eq!(kash_redis(1), Ok(1));
    assert_eq!(kash_redis(5), Err(TestError::Count(5)));
    assert_eq!(kash_redis(6), Err(TestError::Count(6)));
}

#[kash(redis)]
fn kash_redis_cache_create(n: u32) -> Result<u32, TestError> {
    if n < 5 {
        Ok(n)
    } else {
        Err(TestError::Count(n))
    }
}

#[test]
fn test_kash_redis_cache_create() {
    assert_eq!(kash_redis_cache_create(1), Ok(1));
    assert_eq!(kash_redis_cache_create(1), Ok(1));
    assert_eq!(kash_redis_cache_create(5), Err(TestError::Count(5)));
    assert_eq!(kash_redis_cache_create(6), Err(TestError::Count(6)));
}

#[cfg(any(feature = "redis_async_std", feature = "redis_tokio"))]
mod async_redis_tests {
    use super::*;

    #[kash(
        redis(prefix_block = "{ \"__kash_redis_proc_macro_test_fn_async_kash_redis:\" }"),
        ttl = "1"
    )]
    async fn async_kash_redis(n: u32) -> Result<u32, TestError> {
        if n < 5 {
            Ok(n)
        } else {
            Err(TestError::Count(n))
        }
    }

    #[tokio::test]
    async fn test_async_kash_redis() {
        assert_eq!(async_kash_redis(1).await, Ok(1));
        assert_eq!(async_kash_redis(1).await, Ok(1));
        assert_eq!(async_kash_redis(5).await, Err(TestError::Count(5)));
        assert_eq!(async_kash_redis(6).await, Err(TestError::Count(6)));
    }

    #[kash(redis, ttl = "1", name = "async_kash_redis_test_cache_create")]
    async fn async_kash_redis_cache_create(n: u32) -> Result<u32, TestError> {
        if n < 5 {
            Ok(n)
        } else {
            Err(TestError::Count(n))
        }
    }

    #[tokio::test]
    async fn test_async_kash_redis_cache_create() {
        assert_eq!(async_kash_redis_cache_create(1).await, Ok(1));
        assert_eq!(async_kash_redis_cache_create(1).await, Ok(1));
        assert_eq!(
            async_kash_redis_cache_create(5).await,
            Err(TestError::Count(5))
        );
        assert_eq!(
            async_kash_redis_cache_create(6).await,
            Err(TestError::Count(6))
        );
    }
}
