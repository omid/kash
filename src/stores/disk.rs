use crate::IOKash;
use directories::BaseDirs;
use instant::Duration;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sled::Db;
use std::marker::PhantomData;
use std::path::Path;
use std::{path::PathBuf, time::SystemTime};
use thiserror::Error;

pub struct DiskCacheBuilder<K, V> {
    seconds: Option<u64>,
    sync_to_disk_on_cache_change: bool,
    dir: Option<PathBuf>,
    cache_name: String,
    connection_config: Option<sled::Config>,
    _phantom: PhantomData<(K, V)>,
}

#[derive(Error, Debug)]
pub enum DiskCacheBuildError {
    #[error("Storage connection error")]
    ConnectionError(#[from] sled::Error),
    #[error("Connection string not specified or invalid in env var {env_key:?}: {error:?}")]
    MissingPath {
        env_key: String,
        error: std::env::VarError,
    },
}

static DISK_FILE_PREFIX: &str = "kash_disk_cache";
const DISK_FILE_VERSION: u64 = 1;

impl<K, V> DiskCacheBuilder<K, V>
where
    K: ToString,
    V: Serialize + DeserializeOwned,
{
    /// Initialize a `DiskCacheBuilder`
    pub fn new<S: ToString>(cache_name: S) -> Self {
        Self {
            seconds: None,
            sync_to_disk_on_cache_change: false,
            dir: None,
            cache_name: cache_name.to_string(),
            connection_config: None,
            _phantom: Default::default(),
        }
    }

    /// Specify the cache ttl in seconds
    pub fn set_ttl(mut self, seconds: u64) -> Self {
        self.seconds = Some(seconds);
        self
    }

    /// Set the disk path for where the data will be stored
    pub fn set_disk_directory<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.dir = Some(dir.as_ref().into());
        self
    }

    /// Specify whether the cache should sync to disk on each cache change.
    /// [sled] flushes every [sled::Config::flush_every_ms] which has a default value.
    /// In some use cases, the default value may not be quick enough,
    /// or a user may want to reduce the flush rate / turn off auto-flushing to reduce IO (and only flush on cache changes).
    /// (see [DiskCacheBuilder::set_connection_config] for more control over the sled connection)
    pub fn set_sync_to_disk_on_cache_change(mut self, sync_to_disk_on_cache_change: bool) -> Self {
        self.sync_to_disk_on_cache_change = sync_to_disk_on_cache_change;
        self
    }

    /// Specify the [sled::Config] to use for the connection to the disk cache.
    ///
    /// ### Note
    ///
    /// Don't use [sled::Config::path] as any value set here will be overwritten by either
    /// the path specified in [DiskCacheBuilder::set_disk_directory], or the default value calculated by [DiskCacheBuilder].
    ///
    /// ### Example Use Case
    /// By default [sled] automatically syncs to disk at a frequency specified in [sled::Config::flush_every_ms].
    /// A user may want to reduce IO by setting a lower flush frequency, or by setting [sled::Config::flush_every_ms] to [None].
    /// Also see [DiskCacheBuilder::set_sync_to_disk_on_cache_change] which allows for syncing to disk on each cache change.
    /// ```rust
    /// use kash::stores::{DiskCacheBuilder, DiskCache};
    ///
    /// let config = sled::Config::new().flush_every_ms(None);
    /// let cache: DiskCache<String, String> = DiskCacheBuilder::new("my-cache")
    ///     .set_connection_config(config)
    ///     .set_sync_to_disk_on_cache_change(true)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn set_connection_config(mut self, config: sled::Config) -> Self {
        self.connection_config = Some(config);
        self
    }

    fn default_disk_dir() -> PathBuf {
        BaseDirs::new()
            .map(|base_dirs| {
                let exe_name = std::env::current_exe()
                    .ok()
                    .and_then(|path| {
                        dbg!(&path);
                        path.file_name()
                            .and_then(|os_str| os_str.to_str().map(|s| format!("{}_", s)))
                    })
                    .unwrap_or_default();
                let dir_prefix = format!("{}{}", exe_name, DISK_FILE_PREFIX);
                dbg!(&dir_prefix);
                dbg!(&base_dirs.cache_dir());
                dbg!(&base_dirs.cache_dir().join(dir_prefix.clone()));
                base_dirs.cache_dir().join(dir_prefix)
            })
            .unwrap_or_else(|| {
                std::env::current_dir().expect("disk cache unable to determine current directory")
            })
    }

    pub fn build(self) -> Result<DiskCache<K, V>, DiskCacheBuildError> {
        let dir = self.dir.unwrap_or_else(|| Self::default_disk_dir());
        let path = dir.join(format!("{}_v{}", self.cache_name, DISK_FILE_VERSION));
        let connection = match self.connection_config {
            Some(config) => config.path(path.clone()).open()?,
            None => sled::open(path.clone())?,
        };

        Ok(DiskCache {
            seconds: self.seconds,
            sync_to_disk_on_cache_change: self.sync_to_disk_on_cache_change,
            version: DISK_FILE_VERSION,
            path,
            connection,
            _phantom: self._phantom,
        })
    }
}

/// Cache store backed by disk
pub struct DiskCache<K, V> {
    pub(super) seconds: Option<u64>,
    sync_to_disk_on_cache_change: bool,
    #[allow(unused)]
    version: u64,
    #[allow(unused)]
    path: PathBuf,
    connection: Db,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V> DiskCache<K, V>
where
    K: ToString,
    V: Serialize + DeserializeOwned,
{
    #[allow(clippy::new_ret_no_self)]
    /// Initialize a `DiskCacheBuilder`
    pub fn new(cache_name: &str) -> DiskCacheBuilder<K, V> {
        DiskCacheBuilder::new(cache_name)
    }

    pub fn remove_expired_entries(&self) -> Result<(), DiskCacheError> {
        let now = SystemTime::now();

        for (key, value) in self.connection.iter().flatten() {
            if let Ok(kash) = rmp_serde::from_slice::<KashDiskValue<V>>(&value) {
                if let Some(lifetime_seconds) = self.seconds {
                    if now
                        .duration_since(kash.created_at)
                        .unwrap_or(Duration::from_secs(0))
                        >= Duration::from_secs(lifetime_seconds)
                    {
                        self.connection.remove(key)?;
                    }
                }
            }
        }

        if self.sync_to_disk_on_cache_change {
            self.connection.flush()?;
        }
        Ok(())
    }

    /// Provide access to the underlying [Db] connection
    /// This is useful for i.e., manually flushing the cache to disk.
    pub fn connection(&self) -> &Db {
        &self.connection
    }

    /// Provide mutable access to the underlying [Db] connection
    pub fn connection_mut(&mut self) -> &mut Db {
        &mut self.connection
    }
}

#[derive(Error, Debug)]
pub enum DiskCacheError {
    #[error("Storage error")]
    StorageError(#[from] sled::Error),
    #[error("Error deserializing cached value")]
    CacheDeserializationError(#[from] rmp_serde::decode::Error),
    #[error("Error serializing cached value")]
    CacheSerializationError(#[from] rmp_serde::encode::Error),
}

#[derive(serde::Serialize, serde::Deserialize)]
struct KashDiskValue<V> {
    pub(crate) value: V,
    pub(crate) created_at: SystemTime,
    pub(crate) version: u64,
}

impl<V> KashDiskValue<V> {
    fn new(value: V) -> Self {
        Self {
            value,
            created_at: SystemTime::now(),
            version: 1,
        }
    }
}

impl<K, V> IOKash<K, V> for DiskCache<K, V>
where
    K: ToString,
    V: Serialize + DeserializeOwned,
{
    type Error = DiskCacheError;

    fn get(&self, key: &K) -> Result<Option<V>, DiskCacheError> {
        let key = key.to_string();
        let seconds = self.seconds;
        let update = |old: Option<&[u8]>| -> Option<Vec<u8>> {
            let old = old?;
            if seconds.is_none() {
                return Some(old.to_vec());
            }
            let seconds = seconds.unwrap();
            let kash = match rmp_serde::from_slice::<KashDiskValue<V>>(old) {
                Ok(kash) => kash,
                Err(_) => {
                    // unable to deserialize, treat it as not existing
                    return None;
                }
            };
            if SystemTime::now()
                .duration_since(kash.created_at)
                .unwrap_or(Duration::from_secs(0))
                < Duration::from_secs(seconds)
            {
                let cache_val =
                    rmp_serde::to_vec(&kash).expect("error serializing kash disk value");
                Some(cache_val)
            } else {
                None
            }
        };

        if let Some(data) = self.connection.update_and_fetch(key, update)? {
            let kash = rmp_serde::from_slice::<KashDiskValue<V>>(&data)?;
            Ok(Some(kash.value))
        } else {
            Ok(None)
        }
    }

    fn set(&self, key: K, value: V) -> Result<Option<V>, DiskCacheError> {
        let key = key.to_string();
        let value = rmp_serde::to_vec(&KashDiskValue::new(value))?;

        let result = if let Some(data) = self.connection.insert(key, value)? {
            let kash = rmp_serde::from_slice::<KashDiskValue<V>>(&data)?;

            self.check_expiration(kash)
        } else {
            Ok(None)
        };

        if self.sync_to_disk_on_cache_change {
            self.connection.flush()?;
        }

        result
    }

    fn remove(&self, key: &K) -> Result<Option<V>, DiskCacheError> {
        let key = key.to_string();
        let result = if let Some(data) = self.connection.remove(key)? {
            let kash = rmp_serde::from_slice::<KashDiskValue<V>>(&data)?;

            self.check_expiration(kash)
        } else {
            Ok(None)
        };

        if self.sync_to_disk_on_cache_change {
            self.connection.flush()?;
        }

        result
    }

    fn ttl(&self) -> Option<u64> {
        self.seconds
    }

    fn set_ttl(&mut self, seconds: u64) -> Option<u64> {
        let old = self.seconds;
        self.seconds = Some(seconds);
        old
    }

    fn unset_ttl(&mut self) -> Option<u64> {
        self.seconds.take()
    }
}

impl<K, V> DiskCache<K, V>
where
    K: ToString,
    V: DeserializeOwned + Serialize,
{
    fn check_expiration(&self, kash: KashDiskValue<V>) -> Result<Option<V>, DiskCacheError> {
        if let Some(ttl) = self.seconds {
            if SystemTime::now()
                .duration_since(kash.created_at)
                .unwrap_or(Duration::from_secs(0))
                < Duration::from_secs(ttl)
            {
                Ok(Some(kash.value))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(kash.value))
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test_DiskCache {
    use googletest::{
        assert_that,
        matchers::{anything, eq, none, ok, some},
        GoogleTestSupport as _,
    };
    use std::thread::sleep;
    use std::time::Duration;
    use tempfile::TempDir;

    use super::*;

    /// If passing `no_exist` to the macro:
    /// This gives you a TempDir where the directory does not exist
    /// so you can copy / move things to the returned TmpDir.path()
    /// and those files will be removed when the TempDir is dropped
    macro_rules! temp_dir {
        () => {
            TempDir::new().expect("Error creating temp dir")
        };
        (no_exist) => {{
            let tmp_dir = TempDir::new().expect("Error creating temp dir");
            std::fs::remove_dir_all(tmp_dir.path()).expect("error emptying the tmp dir");
            tmp_dir
        }};
    }

    fn now_millis() -> u128 {
        SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    const TEST_KEY: u32 = 1;
    const TEST_VAL: u32 = 100;
    const TEST_KEY_1: u32 = 2;
    const TEST_VAL_1: u32 = 200;
    const LIFE_SPAN_2_SECS: u64 = 2;
    const LIFE_SPAN_1_SEC: u64 = 1;

    #[googletest::test]
    fn cache_get_after_cache_remove_returns_none() {
        let tmp_dir = temp_dir!();
        let cache: DiskCache<u32, u32> = DiskCache::new("test-cache")
            .set_disk_directory(tmp_dir.path())
            .build()
            .unwrap();

        let kash = cache.get(&TEST_KEY).unwrap();
        assert_that!(
            kash,
            none(),
            "Getting a non-existent key-value should return None"
        );

        let kash = cache.set(TEST_KEY, TEST_VAL).unwrap();
        assert_that!(kash, none(), "Setting a new key-value should return None");

        let kash = cache.set(TEST_KEY, TEST_VAL_1).unwrap();
        assert_that!(
            kash,
            some(eq(TEST_VAL)),
            "Setting an existing key-value should return the old value"
        );

        let kash = cache.get(&TEST_KEY).unwrap();
        assert_that!(
            kash,
            some(eq(TEST_VAL_1)),
            "Getting an existing key-value should return the value"
        );

        let kash = cache.remove(&TEST_KEY).unwrap();
        assert_that!(
            kash,
            some(eq(TEST_VAL_1)),
            "Removing an existing key-value should return the value"
        );

        let kash = cache.get(&TEST_KEY).unwrap();
        assert_that!(kash, none(), "Getting a removed key should return None");

        drop(cache);
    }

    #[googletest::test]
    fn values_expire_when_lifespan_elapses_returning_none() {
        let tmp_dir = temp_dir!();
        let cache: DiskCache<u32, u32> = DiskCache::new("test-cache")
            .set_disk_directory(tmp_dir.path())
            .set_ttl(LIFE_SPAN_2_SECS)
            .build()
            .unwrap();

        assert_that!(
            cache.get(&TEST_KEY),
            ok(none()),
            "Getting a non-existent key-value should return None"
        );

        assert_that!(
            cache.set(TEST_KEY, 100),
            ok(none()),
            "Setting a new key-value should return None"
        );
        assert_that!(
            cache.get(&TEST_KEY),
            ok(some(anything())),
            "Getting an existing key-value before it expires should return the value"
        );

        // Let the ttl expire
        sleep(Duration::from_secs(LIFE_SPAN_2_SECS));
        sleep(Duration::from_micros(500)); // a bit extra for good measure
        assert_that!(
            cache.get(&TEST_KEY),
            ok(none()),
            "Getting an expired key-value should return None"
        );
    }

    #[googletest::test]
    fn set_lifespan_to_a_different_lifespan_is_respected() {
        // COPY PASTE of [values_expire_when_lifespan_elapses_returning_none]
        let tmp_dir = temp_dir!();
        let mut cache: DiskCache<u32, u32> = DiskCache::new("test-cache")
            .set_disk_directory(tmp_dir.path())
            .set_ttl(LIFE_SPAN_2_SECS)
            .build()
            .unwrap();

        assert_that!(
            cache.get(&TEST_KEY),
            ok(none()),
            "Getting a non-existent key-value should return None"
        );

        assert_that!(
            cache.set(TEST_KEY, TEST_VAL),
            ok(none()),
            "Setting a new key-value should return None"
        );

        // Let the ttl expire
        sleep(Duration::from_secs(LIFE_SPAN_2_SECS));
        sleep(Duration::from_micros(500)); // a bit extra for good measure
        assert_that!(
            cache.get(&TEST_KEY),
            ok(none()),
            "Getting an expired key-value should return None"
        );

        let old_from_setting_lifespan = cache
            .set_ttl(LIFE_SPAN_1_SEC)
            .expect("error setting new ttl");
        assert_that!(
            old_from_setting_lifespan,
            eq(LIFE_SPAN_2_SECS),
            "Setting ttl should return the old ttl"
        );
        assert_that!(
            cache.set(TEST_KEY, TEST_VAL),
            ok(none()),
            "Setting a previously expired key-value should return None"
        );
        assert_that!(
            cache.get(&TEST_KEY),
            ok(some(eq(&TEST_VAL))),
            "Getting a new set (previously expired) key-value should return the value"
        );

        // Let the new ttl expire
        sleep(Duration::from_secs(LIFE_SPAN_1_SEC));
        sleep(Duration::from_micros(500)); // a bit extra for good measure
        assert_that!(
            cache.get(&TEST_KEY),
            ok(none()),
            "Getting an expired key-value should return None"
        );

        cache.set_ttl(10).expect("error setting ttl");
        assert_that!(
            cache.set(TEST_KEY, TEST_VAL),
            ok(none()),
            "Setting a previously expired key-value should return None"
        );

        // TODO: Why are we now setting an irrelevant key?
        assert_that!(
            cache.set(TEST_KEY_1, TEST_VAL),
            ok(none()),
            "Setting a new, separate, key-value should return None"
        );

        assert_that!(
            cache.get(&TEST_KEY),
            ok(some(eq(&TEST_VAL))),
            "Getting a new set (previously expired) key-value should return the value"
        );
        assert_that!(
            cache.get(&TEST_KEY),
            ok(some(eq(&TEST_VAL))),
            "Getting the same value again should return the value"
        );
    }

    #[googletest::test]
    // TODO: Consider removing this test, as it's not really testing anything.
    // If we want to check that setting a different disk directory to the default doesn't change anything,
    // we should design the tests to run all the same tests but parameterized with different conditions.
    fn does_not_break_when_constructed_using_default_disk_directory() {
        let cache: DiskCache<u32, u32> =
            DiskCache::new(&format!("{}:disk-cache-test-default-dir", now_millis()))
                // use the default disk directory
                .build()
                .unwrap();

        let kash = cache.get(&TEST_KEY).unwrap();
        assert_that!(
            kash,
            none(),
            "Getting a non-existent key-value should return None"
        );

        let kash = cache.set(TEST_KEY, TEST_VAL).unwrap();
        assert_that!(kash, none(), "Setting a new key-value should return None");

        let kash = cache.set(TEST_KEY, TEST_VAL_1).unwrap();
        assert_that!(
            kash,
            some(eq(TEST_VAL)),
            "Setting an existing key-value should return the old value"
        );

        // remove the cache dir to clean up the test as we're not using a temp dir
        std::fs::remove_dir_all(cache.path).expect("error in clean up removing the cache dir")
    }

    mod set_sync_to_disk_on_cache_change {

        mod when_no_auto_flushing {
            use super::super::*;

            fn check_on_recovered_cache(
                set_sync_to_disk_on_cache_change: bool,
                run_on_original_cache: fn(&DiskCache<u32, u32>) -> (),
                run_on_recovered_cache: fn(&DiskCache<u32, u32>) -> (),
            ) {
                let original_cache_tmp_dir = temp_dir!();
                let copied_cache_tmp_dir = temp_dir!(no_exist);
                const CACHE_NAME: &str = "test-cache";

                let cache: DiskCache<u32, u32> = DiskCache::new(CACHE_NAME)
                    .set_disk_directory(original_cache_tmp_dir.path())
                    .set_sync_to_disk_on_cache_change(set_sync_to_disk_on_cache_change) // WHAT'S BEING TESTED
                    // NOTE: disabling automatic flushing, so that we only test the flushing of cache_set
                    .set_connection_config(sled::Config::new().flush_every_ms(None))
                    .build()
                    .unwrap();

                // flush the cache to disk before any cache setting, so that when we create the recovered cache
                // it has something to recover from, even if set_cache doesn't write to disk as we'd like.
                cache
                    .connection
                    .flush()
                    .expect("error flushing cache before any cache setting");

                run_on_original_cache(&cache);

                // freeze the current state of the cache files by copying them to a new location
                // we do this before dropping the cache, as dropping the cache seems to flush to the disk
                let recovered_cache = clone_cache_to_new_location_no_flushing(
                    CACHE_NAME,
                    &cache,
                    copied_cache_tmp_dir.path(),
                );

                assert_that!(recovered_cache.connection.was_recovered(), eq(true));

                run_on_recovered_cache(&recovered_cache);
            }

            mod changes_persist_after_recovery_when_set_to_true {
                use super::*;

                #[googletest::test]
                fn for_cache_set() {
                    check_on_recovered_cache(
                        false,
                        |cache| {
                            // write to the cache, we expect this to persist if the connection is flushed on cache_set
                            cache
                                .set(TEST_KEY, TEST_VAL)
                                .expect("error setting cache in assemble stage");
                        },
                        |recovered_cache| {
                            assert_that!(
                                    recovered_cache.get(&TEST_KEY),
                                    ok(none()),
                                    "set_sync_to_disk_on_cache_change is false, and there is no auto-flushing, so the cache should not have persisted"
                                );
                        },
                    )
                }

                #[googletest::test]
                fn for_cache_remove() {
                    check_on_recovered_cache(
                        false,
                        |cache| {
                            // write to the cache, we expect this to persist if the connection is flushed on cache_set
                            cache
                                .set(TEST_KEY, TEST_VAL)
                                .expect("error setting cache in assemble stage");

                            // manually flush the cache so that we only test cache_remove
                            cache.connection.flush().expect("error flushing cache");

                            cache
                                .remove(&TEST_KEY)
                                .expect("error removing cache in assemble stage");
                        },
                        |recovered_cache| {
                            assert_that!(
                                    recovered_cache.get(&TEST_KEY),
                                    ok(some(eq(&TEST_VAL))),
                                    "set_sync_to_disk_on_cache_change is false, and there is no auto-flushing, so the cache_remove should not have persisted"
                                );
                        },
                    )
                }
            }

            /// This is the anti-test
            mod changes_do_not_persist_after_recovery_when_set_to_false {
                use super::*;

                #[googletest::test]
                fn for_cache_set() {
                    check_on_recovered_cache(
                        true,
                        |cache| {
                            // write to the cache, we expect this to persist if the connection is flushed on cache_set
                            cache
                                .set(TEST_KEY, TEST_VAL)
                                .expect("error setting cache in assemble stage");
                        },
                        |recovered_cache| {
                            assert_that!(
                                recovered_cache.get(&TEST_KEY),
                                ok(some(eq(&TEST_VAL))),
                                "Getting a set key should return the value"
                            );
                        },
                    )
                }

                #[googletest::test]
                fn for_cache_remove() {
                    check_on_recovered_cache(
                        true,
                        |cache| {
                            // write to the cache, we expect this to persist if the connection is flushed on cache_set
                            cache
                                .set(TEST_KEY, TEST_VAL)
                                .expect("error setting cache in assemble stage");

                            cache
                                .remove(&TEST_KEY)
                                .expect("error removing cache in assemble stage");
                        },
                        |recovered_cache| {
                            assert_that!(
                                recovered_cache.get(&TEST_KEY),
                                ok(none()),
                                "Getting a removed key should return None"
                            );
                        },
                    )
                }
            }

            fn clone_cache_to_new_location_no_flushing(
                cache_name: &str,
                cache: &DiskCache<u32, u32>,
                new_location: &Path,
            ) -> DiskCache<u32, u32> {
                copy_dir::copy_dir(cache.path.parent().unwrap(), new_location)
                    .expect("error copying cache files to new location");

                DiskCache::new(cache_name)
                    .set_disk_directory(new_location)
                    .build()
                    .expect("error building cache from copied files")
            }
        }
    }
}
