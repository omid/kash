#![cfg(feature = "disk_store")]

use kash::{kash, DiskCacheError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum TestError {
    #[error("error with disk cache `{0}`")]
    DiskError(String),
    #[error("count `{0}`")]
    Count(u32),
}

impl From<DiskCacheError> for TestError {
    fn from(e: DiskCacheError) -> Self {
        TestError::DiskError(format!("{e:?}"))
    }
}

#[kash(disk, ttl = "1")]
fn kash_disk(n: u32) -> Result<u32, TestError> {
    if n < 5 {
        Ok(n)
    } else {
        Err(TestError::Count(n))
    }
}

#[test]
fn test_kash_disk() {
    assert_eq!(kash_disk(1), Ok(1));
    assert_eq!(kash_disk(1), Ok(1));
    assert_eq!(kash_disk(5), Err(TestError::Count(5)));
    assert_eq!(kash_disk(6), Err(TestError::Count(6)));
}

#[kash(ttl = "1", disk)]
fn kash_disk_cache_create(n: u32) -> Result<u32, TestError> {
    if n < 5 {
        Ok(n)
    } else {
        Err(TestError::Count(n))
    }
}

#[test]
fn test_kash_disk_cache_create() {
    assert_eq!(kash_disk_cache_create(1), Ok(1));
    assert_eq!(kash_disk_cache_create(1), Ok(1));
    assert_eq!(kash_disk_cache_create(5), Err(TestError::Count(5)));
    assert_eq!(kash_disk_cache_create(6), Err(TestError::Count(6)));
}

/// Just calling the macro with connection_config to test, it doesn't break with an expected string
/// for connection_config.
/// There are no simple tests to test this here
#[kash(disk(connection_config = r#"sled::Config::new().flush_every_ms(None)"#))]
fn kash_disk_connection_config(n: u32) -> Result<u32, TestError> {
    if n < 5 {
        Ok(n)
    } else {
        Err(TestError::Count(n))
    }
}

/// Just calling the macro with sync_to_disk_on_cache_change to test it doesn't break with an expected value
/// There are no simple tests to test this here
#[kash(disk(sync_to_disk_on_cache_change))]
fn kash_disk_sync_to_disk_on_cache_change(n: u32) -> Result<u32, TestError> {
    if n < 5 {
        Ok(n)
    } else {
        Err(TestError::Count(n))
    }
}

#[cfg(feature = "async")]
mod async_test {
    use super::*;

    #[kash(disk)]
    async fn async_kash_disk(n: u32) -> Result<u32, TestError> {
        if n < 5 {
            Ok(n)
        } else {
            Err(TestError::Count(n))
        }
    }

    #[tokio::test]
    async fn test_async_kash_disk() {
        assert_eq!(async_kash_disk(1).await, Ok(1));
        assert_eq!(async_kash_disk(1).await, Ok(1));
        assert_eq!(async_kash_disk(5).await, Err(TestError::Count(5)));
        assert_eq!(async_kash_disk(6).await, Err(TestError::Count(6)));
    }
}

#[kash(disk, ttl = "1", option)]
fn kash_disk_optional(n: u32) -> Result<Option<u32>, TestError> {
    if n < 5 {
        Ok(Some(n))
    } else {
        Err(TestError::Count(n))
    }
}

#[test]
fn test_kash_disk_optional() {
    assert_eq!(kash_disk_optional(1), Ok(Some(1)));
    assert_eq!(kash_disk_optional(1), Ok(Some(1)));
    assert_eq!(kash_disk_optional(5), Err(TestError::Count(5)));
    assert_eq!(kash_disk_optional(6), Err(TestError::Count(6)));
}
