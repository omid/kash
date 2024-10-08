// #[cfg(not(feature = "async"))]
use std::{borrow::Borrow, cmp::Eq};
// #[cfg(not(feature = "async"))]
use std::{hash::Hash, time::Duration};

// #[cfg(feature = "async")]
// use moka::future::Cache;
use moka::Entry;
// #[cfg(not(feature = "async"))]
use moka::sync::Cache;

/// Memory Cache
///
/// Stores a limited number of values,
/// evicting expired and least-used entries.
/// Time expiration is determined based on entry insertion time..
/// The TTL of an entry is not updated when retrieved.
///
/// Note: This cache is in-memory only
#[derive(Clone, Debug)]
pub struct MemoryCache<K, V>
where
    K: Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub(super) cache: Cache<K, V>,
}

// #[cfg(not(feature = "async"))]
impl<K: Hash + Eq + Send + Sync + 'static, V: Clone + Send + Sync + 'static> MemoryCache<K, V> {
    /// Creates a new `Cache` with a given size limit and pre-allocated backing data.
    /// Also set if the ttl should be refreshed on retrieving
    #[must_use]
    pub fn new(cache: Cache<K, V>) -> MemoryCache<K, V> {
        MemoryCache { cache }
    }

    /// Returns a reference to the cache's `store`
    #[must_use]
    pub fn get_inner(&self) -> &Cache<K, V> {
        &self.cache
    }

    fn get<Q>(&self, k: &Q) -> Option<Entry<K, V>>
    where
        K: Borrow<Q>,
        Q: ToOwned<Owned = K> + Hash + Eq + ?Sized,
    {
        self.cache
            .entry_by_ref(k)
            .or_optionally_insert_with(|| None)
    }

    fn get_or_set(&self, k: K, v: V) -> Entry<K, V>
    where
        K: Hash + Eq,
    {
        self.cache.entry(k).or_insert(v)
    }

    fn set(&self, k: K, v: V) -> Option<V> {
        let old = self.cache.get(&k);
        self.cache.insert(k, v);
        old
    }

    fn remove(&self, k: &K) -> Option<V> {
        self.cache.remove(k)
    }

    fn reset(&self) {
        self.cache.invalidate_all();
    }

    fn size(&self) -> u64 {
        self.cache.entry_count()
    }

    fn ttl(&mut self) -> Option<Duration> {
        self.cache.policy().time_to_live()
    }
}

#[cfg(test)]
/// Cache store tests
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;

    // #[test]
    // fn timed_sized_cache() {
    //     let mut c = MemoryCache::with_size_and_lifespan(5, 2);
    //     assert!(c.get(&1).is_none());
    //     let misses = c.cache_misses().unwrap();
    //     assert_eq!(1, misses);

    //     assert_eq!(c.set(1, 100), None);
    //     assert!(c.get(&1).is_some());
    //     let hits = c.cache_hits().unwrap();
    //     let misses = c.cache_misses().unwrap();
    //     assert_eq!(1, hits);
    //     assert_eq!(1, misses);

    //     assert_eq!(c.set(2, 100), None);
    //     assert_eq!(c.set(3, 100), None);
    //     assert_eq!(c.set(4, 100), None);
    //     assert_eq!(c.set(5, 100), None);

    //     assert_eq!(c.key_order().copied().collect::<Vec<_>>(), [5, 4, 3, 2, 1]);

    //     sleep(Duration::new(1, 0));

    //     assert_eq!(c.set(6, 100), None);
    //     assert_eq!(c.set(7, 100), None);

    //     assert_eq!(c.key_order().copied().collect::<Vec<_>>(), [7, 6, 5, 4, 3]);

    //     assert!(c.get(&2).is_none());
    //     assert!(c.get(&3).is_some());

    //     assert_eq!(c.key_order().copied().collect::<Vec<_>>(), [3, 7, 6, 5, 4]);

    //     assert_eq!(2, c.cache_misses().unwrap());
    //     assert_eq!(5, c.cache_size());

    //     sleep(Duration::new(1, 0));

    //     assert!(c.get(&1).is_none());
    //     assert!(c.get(&2).is_none());
    //     assert!(c.get(&3).is_none());
    //     assert!(c.get(&4).is_none());
    //     assert!(c.get(&5).is_none());
    //     assert!(c.get(&6).is_some());
    //     assert!(c.get(&7).is_some());

    //     assert_eq!(7, c.cache_misses().unwrap());

    //     assert!(c.set(1, 100).is_none());
    //     assert!(c.set(2, 100).is_none());
    //     assert!(c.set(3, 100).is_none());
    //     assert_eq!(c.key_order().copied().collect::<Vec<_>>(), [3, 2, 1, 7, 6]);

    //     sleep(Duration::new(1, 0));

    //     assert!(c.get(&1).is_some());
    //     assert!(c.get(&2).is_some());
    //     assert!(c.get(&3).is_some());
    //     assert!(c.get(&4).is_none());
    //     assert!(c.get(&5).is_none());
    //     assert!(c.get(&6).is_none());
    //     assert!(c.get(&7).is_none());

    //     assert_eq!(11, c.cache_misses().unwrap());

    //     let mut c = MemoryCache::with_size_and_lifespan(5, 0);
    //     let mut ticker = 0;
    //     let setter = || {
    //         let v = ticker;
    //         ticker += 1;
    //         v
    //     };
    //     assert_eq!(c.cache_get_or_set_with(1, setter), &0);
    //     let setter = || {
    //         let v = ticker;
    //         ticker += 1;
    //         v
    //     };
    //     assert_eq!(c.cache_get_or_set_with(1, setter), &1);
    // }

    // #[test]
    // fn timed_cache_refresh() {
    //     let mut c = MemoryCache::with_size_and_lifespan_and_refresh(2, 2, true);
    //     assert!(c.refresh());
    //     assert_eq!(c.get(&1), None);
    //     let misses = c.cache_misses().unwrap();
    //     assert_eq!(1, misses);

    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.get(&1), Some(&100));
    //     let hits = c.cache_hits().unwrap();
    //     let misses = c.cache_misses().unwrap();
    //     assert_eq!(1, hits);
    //     assert_eq!(1, misses);

    //     assert_eq!(c.set(2, 200), None);
    //     assert_eq!(c.get(&2), Some(&200));
    //     sleep(Duration::new(1, 0));
    //     assert_eq!(c.get(&1), Some(&100));
    //     sleep(Duration::new(1, 0));
    //     assert_eq!(c.get(&1), Some(&100));
    //     assert_eq!(c.get(&2), None);
    // }

    // #[test]
    // fn try_new() {
    //     let c: std::io::Result<MemoryCache<i32, i32>> =
    //         MemoryCache::try_with_size_and_lifespan(0, 2);
    //     assert_eq!(c.unwrap_err().raw_os_error(), Some(22));
    // }

    // #[test]
    // fn clear() {
    //     let mut c = MemoryCache::with_size_and_lifespan(3, 3600);

    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.set(2, 200), None);
    //     assert_eq!(c.set(3, 300), None);
    //     c.cache_clear();

    //     assert_eq!(0, c.cache_size());
    // }

    // #[test]
    // fn reset() {
    //     let init_capacity = 1;
    //     let mut c = MemoryCache::with_size_and_lifespan(init_capacity, 100);
    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.set(2, 200), None);
    //     assert_eq!(c.set(3, 300), None);
    //     assert!(init_capacity <= c.store.capacity);

    //     c.cache_reset();
    //     assert!(init_capacity <= c.store.capacity);
    // }

    // #[test]
    // fn remove() {
    //     let mut c = MemoryCache::with_size_and_lifespan(3, 3600);

    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.set(2, 200), None);
    //     assert_eq!(c.set(3, 300), None);

    //     assert_eq!(Some(100), c.remove(&1));
    //     assert_eq!(2, c.cache_size());

    //     assert_eq!(Some(200), c.remove(&2));
    //     assert_eq!(1, c.cache_size());

    //     assert_eq!(None, c.remove(&2));
    //     assert_eq!(1, c.cache_size());

    //     assert_eq!(Some(300), c.remove(&3));
    //     assert_eq!(0, c.cache_size());
    // }

    // #[test]
    // fn remove_expired() {
    //     let mut c = MemoryCache::with_size_and_lifespan(3, 1);

    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.set(1, 200), Some(100));
    //     assert_eq!(c.cache_size(), 1);

    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     assert_eq!(None, c.remove(&1));
    //     assert_eq!(0, c.cache_size());
    // }

    // #[test]
    // fn insert_expired() {
    //     let mut c = MemoryCache::with_size_and_lifespan(3, 1);

    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.set(1, 200), Some(100));
    //     assert_eq!(c.cache_size(), 1);

    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     assert_eq!(1, c.cache_size());
    //     assert_eq!(None, c.set(1, 300));
    //     assert_eq!(1, c.cache_size());
    // }

    // #[test]
    // fn get_expired() {
    //     let mut c = MemoryCache::with_size_and_lifespan(3, 1);

    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.set(1, 200), Some(100));
    //     assert_eq!(c.cache_size(), 1);

    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     // still around until we try to get
    //     assert_eq!(1, c.cache_size());
    //     assert_eq!(None, c.get(&1));
    //     assert_eq!(0, c.cache_size());
    // }

    // #[test]
    // fn get_mut_expired() {
    //     let mut c = MemoryCache::with_size_and_lifespan(3, 1);

    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.set(1, 200), Some(100));
    //     assert_eq!(c.cache_size(), 1);

    //     std::thread::sleep(std::time::Duration::from_secs(1));
    //     // still around until we try to get
    //     assert_eq!(1, c.cache_size());
    //     assert_eq!(None, c.cache_get_mut(&1));
    //     assert_eq!(0, c.cache_size());
    // }

    // #[test]
    // fn flush_expired() {
    //     let mut c = MemoryCache::with_size_and_lifespan(3, 1);

    //     assert_eq!(c.set(1, 100), None);
    //     assert_eq!(c.set(1, 200), Some(100));
    //     assert_eq!(c.cache_size(), 1);

    //     std::thread::sleep(std::time::Duration::from_secs(2));
    //     // still around until we flush
    //     assert_eq!(1, c.cache_size());
    //     c.flush();
    //     assert_eq!(0, c.cache_size());
    // }

    // #[test]
    // fn get_or_set_with() {
    //     let mut c = MemoryCache::with_size_and_lifespan(5, 2);

    //     assert_eq!(c.cache_get_or_set_with(0, || 0), &0);
    //     assert_eq!(c.cache_get_or_set_with(1, || 1), &1);
    //     assert_eq!(c.cache_get_or_set_with(2, || 2), &2);
    //     assert_eq!(c.cache_get_or_set_with(3, || 3), &3);
    //     assert_eq!(c.cache_get_or_set_with(4, || 4), &4);
    //     assert_eq!(c.cache_get_or_set_with(5, || 5), &5);

    //     assert_eq!(c.cache_misses(), Some(6));

    //     assert_eq!(c.cache_get_or_set_with(0, || 0), &0);

    //     assert_eq!(c.cache_misses(), Some(7));

    //     assert_eq!(c.cache_get_or_set_with(0, || 42), &0);

    //     sleep(Duration::new(1, 0));

    //     assert_eq!(c.cache_get_or_set_with(0, || 42), &0);

    //     assert_eq!(c.cache_get_or_set_with(1, || 1), &1);

    //     assert_eq!(c.cache_get_or_set_with(4, || 42), &4);

    //     assert_eq!(c.cache_get_or_set_with(5, || 42), &5);

    //     assert_eq!(c.cache_get_or_set_with(6, || 6), &6);

    //     assert_eq!(c.cache_misses(), Some(9));

    //     sleep(Duration::new(1, 0));

    //     assert_eq!(c.cache_get_or_set_with(4, || 42), &42);

    //     assert_eq!(c.cache_get_or_set_with(5, || 42), &42);

    //     assert_eq!(c.cache_get_or_set_with(6, || 42), &6);

    //     assert_eq!(c.cache_misses(), Some(11));
    // }

    // #[cfg(feature = "async")]
    // #[tokio::test]
    // async fn test_async_trait_timed_sized() {
    //     let mut c = MemoryCache::with_size_and_lifespan(5, 1);

    //     async fn _get(n: usize) -> usize {
    //         n
    //     }

    //     assert_eq!(c.get_or_set_with(0, || async { _get(0).await }).await, &0);
    //     assert_eq!(c.get_or_set_with(1, || async { _get(1).await }).await, &1);
    //     assert_eq!(c.get_or_set_with(2, || async { _get(2).await }).await, &2);
    //     assert_eq!(c.get_or_set_with(3, || async { _get(3).await }).await, &3);

    //     assert_eq!(c.get_or_set_with(0, || async { _get(3).await }).await, &0);
    //     assert_eq!(c.get_or_set_with(1, || async { _get(3).await }).await, &1);
    //     assert_eq!(c.get_or_set_with(2, || async { _get(3).await }).await, &2);
    //     assert_eq!(c.get_or_set_with(3, || async { _get(1).await }).await, &3);

    //     sleep(Duration::new(1, 0));
    //     // after sleeping, the original val should have expired
    //     assert_eq!(c.get_or_set_with(0, || async { _get(3).await }).await, &3);

    //     c.cache_reset();
    //     async fn _try_get(n: usize) -> Result<usize, String> {
    //         if n < 10 {
    //             Ok(n)
    //         } else {
    //             Err("dead".to_string())
    //         }
    //     }

    //     assert_eq!(
    //         c.try_get_or_set_with(0, || async {
    //             match _try_get(0).await {
    //                 Ok(n) => Ok(n),
    //                 Err(_) => Err("err".to_string()),
    //             }
    //         })
    //         .await
    //         .unwrap(),
    //         &0
    //     );
    //     assert_eq!(
    //         c.try_get_or_set_with(0, || async {
    //             match _try_get(5).await {
    //                 Ok(n) => Ok(n),
    //                 Err(_) => Err("err".to_string()),
    //             }
    //         })
    //         .await
    //         .unwrap(),
    //         &0
    //     );

    //     c.cache_reset();
    //     let res: Result<&mut usize, String> = c
    //         .try_get_or_set_with(0, || async { _try_get(10).await })
    //         .await;
    //     assert!(res.is_err());
    //     assert!(c.key_order().next().is_none());

    //     let res: Result<&mut usize, String> = c
    //         .try_get_or_set_with(0, || async { _try_get(1).await })
    //         .await;
    //     assert_eq!(res.unwrap(), &1);
    //     let res: Result<&mut usize, String> = c
    //         .try_get_or_set_with(0, || async { _try_get(5).await })
    //         .await;
    //     assert_eq!(res.unwrap(), &1);
    //     sleep(Duration::new(1, 0));
    //     let res: Result<&mut usize, String> = c
    //         .try_get_or_set_with(0, || async { _try_get(5).await })
    //         .await;
    //     assert_eq!(res.unwrap(), &5);
    // }
}
