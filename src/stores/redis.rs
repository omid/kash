use crate::IOKash;
use redis::Pipeline;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Display;
use std::marker::PhantomData;
use thiserror::Error;

pub struct RedisCacheBuilder<K, V> {
    seconds: Option<u64>,
    namespace: String,
    prefix: String,
    connection_string: Option<String>,
    pool_max_size: Option<u32>,
    pool_min_idle: Option<u32>,
    pool_max_lifetime: Option<std::time::Duration>,
    pool_idle_timeout: Option<std::time::Duration>,
    _phantom: PhantomData<(K, V)>,
}

const ENV_KEY: &str = "KASH_REDIS_CONNECTION_STRING";
const DEFAULT_NAMESPACE: &str = "kash:";

#[derive(Error, Debug)]
pub enum RedisCacheBuildError {
    #[error("redis connection error")]
    Connection(#[from] redis::RedisError),
    #[error("redis pool error")]
    Pool(#[from] r2d2::Error),
    #[error("Connection string not specified or invalid in env var {env_key:?}: {error:?}")]
    MissingConnectionString {
        env_key: String,
        error: std::env::VarError,
    },
}

impl<K, V> RedisCacheBuilder<K, V>
where
    K: Display,
    V: Serialize + DeserializeOwned,
{
    /// Initialize a `RedisCacheBuilder`
    pub fn new<S: ToString>(prefix: S, seconds: Option<u64>) -> RedisCacheBuilder<K, V> {
        Self {
            seconds,
            namespace: DEFAULT_NAMESPACE.to_string(),
            prefix: prefix.to_string(),
            connection_string: None,
            pool_max_size: None,
            pool_min_idle: None,
            pool_max_lifetime: None,
            pool_idle_timeout: None,
            _phantom: PhantomData,
        }
    }

    /// Specify the cache ttl in seconds
    #[must_use]
    pub fn set_ttl(mut self, seconds: u64) -> Self {
        self.seconds = Some(seconds);
        self
    }

    /// Set the namespace for cache keys. Defaults to `kash:`.
    /// Used to generate keys formatted as: `{namespace}{prefix}{key}`
    /// Note that no delimiters are implicitly added, so you may pass
    /// an empty string if you want there to be no namespace on keys.
    #[must_use]
    pub fn set_namespace<S: ToString>(mut self, namespace: S) -> Self {
        self.namespace = namespace.to_string();
        self
    }

    /// Set the prefix for cache keys.
    /// Used to generate keys formatted as: `{namespace}{prefix}{key}`
    /// Note that no delimiters are implicitly added, so you may pass
    /// an empty string if you want there to be no prefix on keys.
    #[must_use]
    pub fn set_prefix<S: ToString>(mut self, prefix: S) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    /// Set the connection string for redis
    #[must_use]
    pub fn set_connection_string(mut self, cs: &str) -> Self {
        self.connection_string = Some(cs.to_string());
        self
    }

    /// Set the max size of the underlying redis connection pool
    #[must_use]
    pub fn set_connection_pool_max_size(mut self, max_size: u32) -> Self {
        self.pool_max_size = Some(max_size);
        self
    }

    /// Set the minimum number of idle redis connections that should be maintained by the
    /// underlying redis connection pool
    #[must_use]
    pub fn set_connection_pool_min_idle(mut self, min_idle: u32) -> Self {
        self.pool_min_idle = Some(min_idle);
        self
    }

    /// Set the max lifetime of connections used by the underlying redis connection pool
    #[must_use]
    pub fn set_connection_pool_max_lifetime(mut self, max_lifetime: std::time::Duration) -> Self {
        self.pool_max_lifetime = Some(max_lifetime);
        self
    }

    /// Set the max lifetime of idle connections maintained by the underlying redis connection pool
    #[must_use]
    pub fn set_connection_pool_idle_timeout(mut self, idle_timeout: std::time::Duration) -> Self {
        self.pool_idle_timeout = Some(idle_timeout);
        self
    }

    /// Return the current connection string or load from the env var: `KASH_REDIS_CONNECTION_STRING`
    ///
    /// # Errors
    ///
    /// Will return `RedisCacheBuildError::MissingConnectionString` if connection string is not set
    pub fn connection_string(&self) -> Result<String, RedisCacheBuildError> {
        match self.connection_string {
            Some(ref s) => Ok(s.to_string()),
            None => {
                std::env::var(ENV_KEY).map_err(|e| RedisCacheBuildError::MissingConnectionString {
                    env_key: ENV_KEY.to_string(),
                    error: e,
                })
            }
        }
    }

    fn create_pool(&self) -> Result<r2d2::Pool<redis::Client>, RedisCacheBuildError> {
        let s = self.connection_string()?;
        let client: redis::Client = redis::Client::open(s)?;
        // some pool-builder defaults are set when the builder is initialized
        // so we can't overwrite any values with Nones...
        let pool_builder = r2d2::Pool::builder();
        let pool_builder = if let Some(max_size) = self.pool_max_size {
            pool_builder.max_size(max_size)
        } else {
            pool_builder
        };
        let pool_builder = if let Some(min_idle) = self.pool_min_idle {
            pool_builder.min_idle(Some(min_idle))
        } else {
            pool_builder
        };
        let pool_builder = if let Some(max_lifetime) = self.pool_max_lifetime {
            pool_builder.max_lifetime(Some(max_lifetime))
        } else {
            pool_builder
        };
        let pool_builder = if let Some(idle_timeout) = self.pool_idle_timeout {
            pool_builder.idle_timeout(Some(idle_timeout))
        } else {
            pool_builder
        };

        let pool: r2d2::Pool<redis::Client> = pool_builder.build(client)?;
        Ok(pool)
    }

    /// The last step in building a `RedisCache` is to call `build()`
    ///
    /// # Errors
    ///
    /// Will return a `RedisCacheBuildError`, depending on the error
    pub fn build(self) -> Result<RedisCache<K, V>, RedisCacheBuildError> {
        Ok(RedisCache {
            seconds: self.seconds,
            connection_string: self.connection_string()?,
            pool: self.create_pool()?,
            namespace: self.namespace,
            prefix: self.prefix,
            _phantom: PhantomData,
        })
    }
}

/// Cache store backed by redis
///
/// Values have a ttl applied and enforced by redis.
/// Uses an r2d2 connection pool under the hood.
pub struct RedisCache<K, V> {
    pub(super) seconds: Option<u64>,
    pub(super) namespace: String,
    pub(super) prefix: String,
    connection_string: String,
    pool: r2d2::Pool<redis::Client>,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> RedisCache<K, V>
where
    K: Display,
    V: Serialize + DeserializeOwned,
{
    #[allow(clippy::new_ret_no_self)]
    /// Initialize a `RedisCacheBuilder`
    pub fn new<S: ToString>(prefix: S, seconds: Option<u64>) -> RedisCacheBuilder<K, V> {
        RedisCacheBuilder::new(prefix, seconds)
    }

    fn generate_key(&self, key: &K) -> String {
        format!("{}{}{}", self.namespace, self.prefix, key)
    }

    /// Return the redis connection string used
    #[must_use]
    pub fn connection_string(&self) -> String {
        self.connection_string.clone()
    }
}

#[derive(Error, Debug)]
pub enum RedisCacheError {
    #[error("redis error")]
    RedisCacheError(#[from] redis::RedisError),
    #[error("redis pool error")]
    PoolError(#[from] r2d2::Error),
    #[error("Error deserializing cached value")]
    CacheDeserializationError(#[from] rmp_serde::decode::Error),
    #[error("Error serializing cached value")]
    CacheSerializationError(#[from] rmp_serde::encode::Error),
}

impl<K, V> IOKash<K, V> for RedisCache<K, V>
where
    K: Display,
    V: Serialize + DeserializeOwned,
{
    type Error = RedisCacheError;

    fn get(&self, key: &K) -> Result<Option<V>, RedisCacheError> {
        let mut conn = self.pool.get()?;
        let mut pipe = redis::pipe();
        let key = self.generate_key(key);

        pipe.get(&key);
        // ugh: https://github.com/mitsuhiko/redis-rs/pull/388#issuecomment-910919137
        let res: (Option<Vec<u8>>,) = pipe.query(&mut *conn)?;
        check_and_get_result(res)
    }

    fn set(&self, key: K, val: V) -> Result<Option<V>, RedisCacheError> {
        let mut conn = self.pool.get()?;
        let mut pipe = redis::pipe();
        let key = self.generate_key(&key);

        pipe.get(&key);
        let val = rmp_serde::to_vec(&val)?;
        set_val(self.seconds, &mut pipe, key, &val);

        let res: (Option<Vec<u8>>,) = pipe.query(&mut *conn)?;
        check_and_get_result(res)
    }

    fn remove(&self, key: &K) -> Result<Option<V>, RedisCacheError> {
        let mut conn = self.pool.get()?;
        let mut pipe = redis::pipe();
        let key = self.generate_key(key);

        pipe.get(&key);
        pipe.del(key).ignore();
        let res: (Option<Vec<u8>>,) = pipe.query(&mut *conn)?;
        check_and_get_result(res)
    }

    fn ttl(&self) -> Option<u64> {
        self.seconds
    }

    fn set_ttl(&mut self, seconds: u64) -> Option<u64> {
        let old = self.seconds;
        self.seconds = Some(seconds);
        old
    }
}

#[cfg(all(
    feature = "async",
    any(feature = "redis_async_std", feature = "redis_tokio")
))]
mod async_redis {
    use super::{
        check_and_get_result, set_val, DeserializeOwned, Display, PhantomData,
        RedisCacheBuildError, RedisCacheError, Serialize, DEFAULT_NAMESPACE, ENV_KEY,
    };
    use crate::IOKashAsync;

    pub struct AsyncRedisCacheBuilder<K, V> {
        seconds: Option<u64>,
        namespace: String,
        prefix: String,
        connection_string: Option<String>,
        _phantom: PhantomData<(K, V)>,
    }

    impl<K, V> AsyncRedisCacheBuilder<K, V>
    where
        K: Display,
        V: Serialize + DeserializeOwned,
    {
        /// Initialize a `RedisCacheBuilder`
        pub fn new<S: ToString>(prefix: S, seconds: Option<u64>) -> AsyncRedisCacheBuilder<K, V> {
            Self {
                seconds,
                namespace: DEFAULT_NAMESPACE.to_string(),
                prefix: prefix.to_string(),
                connection_string: None,
                _phantom: PhantomData,
            }
        }

        /// Specify the cache ttl in seconds
        #[must_use]
        pub fn set_ttl(mut self, seconds: Option<u64>) -> Self {
            self.seconds = seconds;
            self
        }

        /// Set the namespace for cache keys. Defaults to `kash:`.
        /// Used to generate keys formatted as: `{namespace}{prefix}{key}`
        /// Note that no delimiters are implicitly added, so you may pass
        /// an empty string if you want there to be no namespace on keys.
        #[must_use]
        pub fn set_namespace<S: ToString>(mut self, namespace: S) -> Self {
            self.namespace = namespace.to_string();
            self
        }

        /// Set the prefix for cache keys
        /// Used to generate keys formatted as: `{namespace}{prefix}{key}`
        /// Note that no delimiters are implicitly added, so you may pass
        /// an empty string if you want there to be no prefix on keys.
        #[must_use]
        pub fn set_prefix<S: ToString>(mut self, prefix: S) -> Self {
            self.prefix = prefix.to_string();
            self
        }

        /// Set the connection string for redis
        #[must_use]
        pub fn set_connection_string(mut self, cs: &str) -> Self {
            self.connection_string = Some(cs.to_string());
            self
        }

        /// Return the current connection string or load from the env var: `KASH_REDIS_CONNECTION_STRING`
        ///
        /// # Errors
        ///
        /// Will return `RedisCacheBuildError::MissingConnectionString` if connection string is not set
        pub fn connection_string(&self) -> Result<String, RedisCacheBuildError> {
            match self.connection_string {
                Some(ref s) => Ok(s.to_string()),
                None => std::env::var(ENV_KEY).map_err(|e| {
                    RedisCacheBuildError::MissingConnectionString {
                        env_key: ENV_KEY.to_string(),
                        error: e,
                    }
                }),
            }
        }

        /// Create a multiplexed redis connection. This is a single connection that can
        /// be used asynchronously by multiple futures.
        #[cfg(not(feature = "redis_connection_manager"))]
        async fn create_multiplexed_connection(
            &self,
        ) -> Result<redis::aio::MultiplexedConnection, RedisCacheBuildError> {
            let s = self.connection_string()?;
            let client = redis::Client::open(s)?;
            let conn = client.get_multiplexed_async_connection().await?;
            Ok(conn)
        }

        /// Create a multiplexed connection wrapped in a manager. The manager provides access
        /// to a multiplexed connection and will automatically reconnect to the server when
        /// necessary.
        #[cfg(feature = "redis_connection_manager")]
        async fn create_connection_manager(
            &self,
        ) -> Result<redis::aio::ConnectionManager, RedisCacheBuildError> {
            let s = self.connection_string()?;
            let client = redis::Client::open(s)?;
            let conn = redis::aio::ConnectionManager::new(client).await?;
            Ok(conn)
        }

        /// The last step in building a `RedisCache` is to call `build()`
        ///
        /// # Errors
        ///
        /// Will return a `RedisCacheBuildError`, depending on the error
        pub async fn build(self) -> Result<AsyncRedisCache<K, V>, RedisCacheBuildError> {
            Ok(AsyncRedisCache {
                seconds: self.seconds,
                connection_string: self.connection_string()?,
                #[cfg(not(feature = "redis_connection_manager"))]
                connection: self.create_multiplexed_connection().await?,
                #[cfg(feature = "redis_connection_manager")]
                connection: self.create_connection_manager().await?,
                namespace: self.namespace,
                prefix: self.prefix,
                _phantom: PhantomData,
            })
        }
    }

    /// Cache store backed by redis
    ///
    /// Values have a ttl applied and enforced by redis.
    /// Uses a `redis::aio::MultiplexedConnection` or `redis::aio::ConnectionManager`
    /// under the hood depending on if feature `redis_connection_manager` is used or not.
    pub struct AsyncRedisCache<K, V> {
        pub(super) seconds: Option<u64>,
        pub(super) namespace: String,
        pub(super) prefix: String,
        connection_string: String,
        #[cfg(not(feature = "redis_connection_manager"))]
        connection: redis::aio::MultiplexedConnection,
        #[cfg(feature = "redis_connection_manager")]
        connection: redis::aio::ConnectionManager,
        _phantom: PhantomData<(K, V)>,
    }

    impl<K, V> AsyncRedisCache<K, V>
    where
        K: Display + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync,
    {
        #[allow(clippy::new_ret_no_self)]
        /// Initialize an `AsyncRedisCacheBuilder`
        pub fn new<S: ToString>(prefix: S, seconds: Option<u64>) -> AsyncRedisCacheBuilder<K, V> {
            AsyncRedisCacheBuilder::new(prefix, seconds)
        }

        fn generate_key(&self, key: &K) -> String {
            format!("{}{}{}", self.namespace, self.prefix, key)
        }

        /// Return the redis connection string used
        #[must_use]
        pub fn connection_string(&self) -> &str {
            &self.connection_string
        }
    }

    #[async_trait::async_trait]
    impl<K, V> IOKashAsync<K, V> for AsyncRedisCache<K, V>
    where
        K: Display + Send + Sync,
        V: Serialize + DeserializeOwned + Send + Sync,
    {
        type Error = RedisCacheError;

        /// Get a cached value
        async fn get(&self, key: &K) -> Result<Option<V>, Self::Error> {
            let mut conn = self.connection.clone();
            let mut pipe = redis::pipe();
            let key = self.generate_key(key);

            pipe.get(&key);
            let res: (Option<Vec<u8>>,) = pipe.query_async(&mut conn).await?;
            check_and_get_result(res)
        }

        /// Set a cached value
        async fn set(&self, key: K, val: V) -> Result<Option<V>, Self::Error> {
            let mut conn = self.connection.clone();
            let mut pipe = redis::pipe();
            let key = self.generate_key(&key);

            pipe.get(&key);
            let val = rmp_serde::to_vec(&val)?;
            set_val(self.seconds, &mut pipe, key, &val);

            let res: (Option<Vec<u8>>,) = pipe.query_async(&mut conn).await?;
            check_and_get_result(res)
        }

        /// Remove a cached value
        async fn remove(&self, key: &K) -> Result<Option<V>, Self::Error> {
            let mut conn = self.connection.clone();
            let mut pipe = redis::pipe();
            let key = self.generate_key(key);

            pipe.get(&key);
            pipe.del(&key).ignore();
            let res: (Option<Vec<u8>>,) = pipe.query_async(&mut conn).await?;
            check_and_get_result(res)
        }

        /// Return the ttl of cached values (time to eviction)
        fn ttl(&self) -> Option<u64> {
            self.seconds
        }

        /// Set the ttl of cached values, returns the old value
        fn set_ttl(&mut self, seconds: u64) -> Option<u64> {
            let old = self.seconds;
            self.seconds = Some(seconds);
            old
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::thread::sleep;
        use std::time::Duration;

        fn now_millis() -> u128 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        }

        #[tokio::test]
        async fn test_async_redis_cache() {
            let mut c: AsyncRedisCache<u32, u32> =
                AsyncRedisCache::new(format!("{}:async-redis-cache-test", now_millis()), Some(2))
                    .build()
                    .await
                    .unwrap();

            assert!(c.get(&1).await.unwrap().is_none());

            assert!(c.set(1, 100).await.unwrap().is_none());
            assert!(c.get(&1).await.unwrap().is_some());

            sleep(Duration::new(2, 500_000));
            assert!(c.get(&1).await.unwrap().is_none());

            let old = c.set_ttl(1).unwrap();
            assert_eq!(2, old);
            assert!(c.set(1, 100).await.unwrap().is_none());
            assert!(c.get(&1).await.unwrap().is_some());

            sleep(Duration::new(1, 600_000));
            assert!(c.get(&1).await.unwrap().is_none());

            c.set_ttl(10).unwrap();
            assert!(c.set(1, 100).await.unwrap().is_none());
            assert!(c.set(2, 100).await.unwrap().is_none());
            assert_eq!(c.get(&1).await.unwrap().unwrap(), 100);
            assert_eq!(c.get(&1).await.unwrap().unwrap(), 100);
        }
    }
}

fn check_and_get_result<V>(res: (Option<Vec<u8>>,)) -> Result<Option<V>, RedisCacheError>
where
    V: Serialize + DeserializeOwned,
{
    match res.0 {
        None => Ok(None),
        Some(s) => {
            let v = rmp_serde::from_slice(&s)?;
            Ok(Some(v))
        }
    }
}

fn set_val(seconds: Option<u64>, pipe: &mut Pipeline, key: String, val: &[u8]) {
    if let Some(seconds) = seconds {
        pipe.set_ex(key, val, seconds).ignore();
    } else {
        pipe.set(key, val).ignore();
    }
}

#[cfg(all(
    feature = "async",
    any(feature = "redis_async_std", feature = "redis_tokio")
))]
#[cfg_attr(
    docsrs,
    doc(cfg(all(
        feature = "async",
        any(feature = "redis_async_std", feature = "redis_tokio")
    )))
)]
pub use async_redis::{AsyncRedisCache, AsyncRedisCacheBuilder};

#[cfg(test)]
/// Cache store tests
mod tests {
    use std::thread::sleep;
    use std::time::Duration;

    use super::*;

    fn now_millis() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    #[test]
    fn redis_cache() {
        let mut c: RedisCache<u32, u32> =
            RedisCache::new(format!("{}:redis-cache-test", now_millis()), Some(2))
                .set_namespace("in-tests:")
                .build()
                .unwrap();

        assert!(c.get(&1).unwrap().is_none());

        assert!(c.set(1, 100).unwrap().is_none());
        assert!(c.get(&1).unwrap().is_some());

        sleep(Duration::new(2, 500_000));
        assert!(c.get(&1).unwrap().is_none());

        let old = c.set_ttl(1).unwrap();
        assert_eq!(2, old);
        assert!(c.set(1, 100).unwrap().is_none());
        assert!(c.get(&1).unwrap().is_some());

        sleep(Duration::new(1, 600_000));
        assert!(c.get(&1).unwrap().is_none());

        c.set_ttl(10).unwrap();
        assert!(c.set(1, 100).unwrap().is_none());
        assert!(c.set(2, 100).unwrap().is_none());
        assert_eq!(c.get(&1).unwrap().unwrap(), 100);
        assert_eq!(c.get(&1).unwrap().unwrap(), 100);
    }

    #[test]
    fn remove() {
        let c: RedisCache<u32, u32> = RedisCache::new(
            format!("{}:redis-cache-test-remove", now_millis()),
            Some(3600),
        )
        .build()
        .unwrap();

        assert!(c.set(1, 100).unwrap().is_none());
        assert!(c.set(2, 200).unwrap().is_none());
        assert!(c.set(3, 300).unwrap().is_none());

        assert_eq!(100, c.remove(&1).unwrap().unwrap());
    }
}
