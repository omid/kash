#![cfg(feature = "redis_store")]

use kash::proc_macro::io_kash;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum TestError {
    #[error("error with redis cache `{0}`")]
    RedisError(String),
    #[error("count `{0}`")]
    Count(u32),
}

#[io_kash(
    redis,
    ttl = 1,
    cache_prefix_block = "{ \"__kash_redis_proc_macro_test_fn_kash_redis\" }",
    map_error = r##"|e| TestError::RedisError(format!("{:?}", e))"##
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

#[io_kash(
    redis,
    ttl = 1,
    wrap_return,
    map_error = r##"|e| TestError::RedisError(format!("{:?}", e))"##
)]
fn kash_redis_flag(n: u32) -> Result<kash::Return<u32>, TestError> {
    if n < 5 {
        Ok(kash::Return::new(n))
    } else {
        Err(TestError::Count(n))
    }
}

#[test]
fn test_kash_redis_flag() {
    assert!(!kash_redis_flag(1).unwrap().was_cached);
    assert!(kash_redis_flag(1).unwrap().was_cached);
    assert!(kash_redis_flag(5).is_err());
    assert!(kash_redis_flag(6).is_err());
}

#[io_kash(
    map_error = r##"|e| TestError::RedisError(format!("{:?}", e))"##,
    redis
)]
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

    #[io_kash(
        redis,
        ttl = 1,
        cache_prefix_block = "{ \"__kash_redis_proc_macro_test_fn_async_kash_redis\" }",
        map_error = r##"|e| TestError::RedisError(format!("{:?}", e))"##
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

    #[io_kash(
        redis,
        ttl = 1,
        wrap_return,
        map_error = r##"|e| TestError::RedisError(format!("{:?}", e))"##
    )]
    async fn async_kash_redis_flag(n: u32) -> Result<kash::Return<u32>, TestError> {
        if n < 5 {
            Ok(kash::Return::new(n))
        } else {
            Err(TestError::Count(n))
        }
    }

    #[tokio::test]
    async fn test_async_kash_redis_flag() {
        assert!(!async_kash_redis_flag(1).await.unwrap().was_cached);
        assert!(async_kash_redis_flag(1).await.unwrap().was_cached,);
        assert!(async_kash_redis_flag(5).await.is_err());
        assert!(async_kash_redis_flag(6).await.is_err());
    }

    #[io_kash(
        map_error = r##"|e| TestError::RedisError(format!("{:?}", e))"##,
        redis,
        ttl = "1",
        name = "async_kash_redis_test_cache_create"
    )]
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
