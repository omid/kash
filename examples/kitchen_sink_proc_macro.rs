use kash::proc_macro::kash;
use kash::Return;
use kash::{Kash, SizedCache, UnboundCache};
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::thread::{sleep, spawn};
use std::time::Duration;

// kash shorthand, uses the default unbounded cache.
// Equivalent to specifying `ty = "UnboundCache<(u32), u32>", create= "{ UnboundCache::new() }"`
#[kash]
fn fib(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib(n - 1) + fib(n - 2)
}

#[kash(name = "FLIB")]
fn fib_2(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib(n - 1) + fib(n - 2)
}

// Same as above, but preallocates some space.
#[kash(
    ty = "UnboundCache<u32, u32>",
    create = "{ UnboundCache::with_capacity(50) }"
)]
fn fib_specific(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib_specific(n - 1) + fib_specific(n - 2)
}

// Specify a specific cache type
// Note that the cache key type is a tuple of function argument types.
#[kash(
    ty = "SizedCache<(u32, u32), u32>",
    create = "{ SizedCache::with_size(100) }"
)]
fn slow(a: u32, b: u32) -> u32 {
    sleep(Duration::new(2, 0));
    a * b
}

// Specify a specific cache type and an explicit key expression
// Note that the cache key type is a `String` created from the borrow arguments
// Note that key is not used, convert requires either key or type to be set.
#[kash(
    ty = "SizedCache<String, usize>",
    create = "{ SizedCache::with_size(100) }",
    convert = r#"{ format!("{}{}", a, b) }"#
)]
fn keyed(a: &str, b: &str) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}

#[kash(key = "String", convert = r#"{ format!("{}{}", a, b) }"#)]
fn keyed_key(a: &str, b: &str) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}

// Implement our own cache type
struct MyCache<K: Hash + Eq, V> {
    store: HashMap<K, V>,
    capacity: usize,
}
impl<K: Hash + Eq, V> MyCache<K, V> {
    pub fn with_capacity(size: usize) -> MyCache<K, V> {
        MyCache {
            store: HashMap::with_capacity(size),
            capacity: size,
        }
    }
}
impl<K: Hash + Eq, V> Kash<K, V> for MyCache<K, V> {
    fn cache_get<Q>(&mut self, k: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.store.get(k)
    }
    fn cache_get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.store.get_mut(k)
    }
    fn cache_get_or_set_with<F: FnOnce() -> V>(&mut self, k: K, f: F) -> &mut V {
        self.store.entry(k).or_insert_with(f)
    }
    fn cache_set(&mut self, k: K, v: V) -> Option<V> {
        self.store.insert(k, v)
    }
    fn cache_remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.store.remove(k)
    }
    fn cache_clear(&mut self) {
        self.store.clear();
    }
    fn cache_reset(&mut self) {
        self.store = HashMap::with_capacity(self.capacity);
    }
    fn cache_size(&self) -> usize {
        self.store.len()
    }
}

// Specify our custom cache and supply an instance to use
#[kash(ty = "MyCache<u32, ()>", create = "{ MyCache::with_capacity(50) }")]
fn custom(n: u32) {
    if n == 0 {
        return;
    }
    custom(n - 1);
}

// handle results, don't cache errors
#[kash(result = true)]
fn slow_result(a: u32, b: u32) -> Result<u32, ()> {
    sleep(Duration::new(2, 0));
    Ok(a * b)
}

// return a flag indicated whether the result was cached
#[kash(wrap_return = true)]
fn wrap_return(a: String) -> Return<String> {
    sleep(Duration::new(1, 0));
    Return::new(a)
}

// return a flag indicated whether the result was cached, with a result type
#[kash(result = true, wrap_return = true)]
fn wrap_return_result(a: String) -> Result<kash::Return<String>, ()> {
    sleep(Duration::new(1, 0));
    Ok(Return::new(a))
}

// return a flag indicated whether the result was cached, with an option type
#[kash(option = true, wrap_return = true)]
fn wrap_return_option(a: String) -> Option<Return<String>> {
    sleep(Duration::new(1, 0));
    Some(Return::new(a))
}

// A simple cache that expires after a second. We'll keep the
// value fresh by priming it in a separate thread.
#[kash(time = 1)]
fn expires_for_priming(a: i32) -> i32 {
    a
}

// NOTE:
// The following fails with compilation error
// ```
//   error:
//   When specifying `wrap_return = true`, the return type must be wrapped in `kash::Return<T>`.
//   The following return types are supported:
//   |    `kash::Return<T>`
//   |    `std::result::Result<kashReturn<T>, E>`
//   |    `std::option::Option<kashReturn<T>>`
//   Found type: std::result::Result<u32,()>.
// ```
//
// #[kash(wrap_return = true)]
// fn wrap_return_requires_return_type(a: u32) -> std::result::Result<u32, ()> {
//     Ok(1)
// }

pub fn main() {
    println!("\n ** default cache with default name **");
    fib(3);
    fib(3);
    {
        let cache = FIB.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits().unwrap(), 2);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses(), Some(4));
        // make sure lock is dropped
    }
    fib(10);
    fib(10);

    println!("\n ** default cache with explicit name **");
    fib_2(3);
    fib_2(3);
    {
        let cache = FLIB.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits().unwrap(), 1);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses(), Some(1));
        // make sure lock is dropped
    }

    println!("\n ** specific cache **");
    fib_specific(20);
    fib_specific(20);
    {
        let cache = FIB_SPECIFIC.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits().unwrap(), 19);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses(), Some(21));
        // make sure lock is dropped
    }
    fib_specific(20);
    fib_specific(20);

    println!("\n ** custom cache **");
    custom(25);
    {
        let cache = CUSTOM.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits(), None);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses(), None);
        //custom cache doesn't implement these so they're None
        // make sure lock is dropped
    }

    println!("\n ** slow func **");
    println!(" - first run `slow(10)`");
    slow(10, 10);
    println!(" - second run `slow(10)`");
    slow(10, 10);
    {
        let cache = SLOW.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits().unwrap(), 1);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses().unwrap(), 1);
        // make sure the cache-lock is dropped
    }

    println!("\n ** slow result func **");
    println!(" - first run `slow_result(10)`");
    let _ = slow_result(10, 10);
    println!(" - second run `slow_result(10)`");
    let _ = slow_result(10, 10);
    {
        let cache = SLOW_RESULT.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits().unwrap(), 1);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses(), Some(1));
        // make sure the cache-lock is dropped
    }

    println!("\n ** with kash flag func **");
    println!(" - first run `wrap_return(\"a\")`");
    let r = wrap_return("a".to_string());
    println!("was cached: {}", r.was_cached);
    println!(" - second run `wrap_return(\"a\")`");
    let r = wrap_return("a".to_string());
    println!("was cached: {}", r.was_cached);
    println!("derefs to inner, *r == \"a\" : {}", *r == "a");
    println!(
        "derefs to inner, r.as_str() == \"a\" : {}",
        r.as_str() == "a"
    );
    {
        let cache = WRAP_RETURN.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits().unwrap(), 1);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses(), Some(1));
        // make sure the cache-lock is dropped
    }

    println!("\n ** with kash flag result func **");
    println!(" - first run `wrap_return_result(\"a\")`");
    let r = wrap_return_result("a".to_string()).expect("wrap_return_result failed");
    println!("was cached: {}", r.was_cached);
    println!(" - second run `wrap_return_result(\"a\")`");
    let r = wrap_return_result("a".to_string()).expect("wrap_return_result failed");
    println!("was cached: {}", r.was_cached);
    println!("derefs to inner, *r : {:?}", *r);
    println!("derefs to inner, *r == \"a\" : {}", *r == "a");
    println!(
        "derefs to inner, r.as_str() == \"a\" : {}",
        r.as_str() == "a"
    );
    {
        let cache = WRAP_RETURN_RESULT.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits().unwrap(), 1);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses(), Some(1));
        // make sure the cache-lock is dropped
    }

    println!("\n ** with kash flag option func **");
    println!(" - first run `wrap_return_option(\"a\")`");
    let r = wrap_return_option("a".to_string()).expect("wrap_return_result failed");
    println!("was cached: {}", r.was_cached);
    println!(" - second run `wrap_return_option(\"a\")`");
    let r = wrap_return_option("a".to_string()).expect("wrap_return_result failed");
    println!("was cached: {}", r.was_cached);
    println!("derefs to inner, *r : {:?}", *r);
    println!("derefs to inner, *r == \"a\" : {}", *r == "a");
    println!(
        "derefs to inner, r.as_str() == \"a\" : {}",
        r.as_str() == "a"
    );
    {
        let cache = WRAP_RETURN_OPTION.lock().unwrap();
        println!("hits: {:?}", cache.cache_hits());
        assert_eq!(cache.cache_hits().unwrap(), 1);
        println!("misses: {:?}", cache.cache_misses());
        assert_eq!(cache.cache_misses(), Some(1));
        // make sure the cache-lock is dropped
    }

    println!("\n ** refresh by priming **");
    let h = spawn(|| {
        for _ in 1..6 {
            expires_for_priming_prime_cache(1);
            sleep(Duration::from_millis(500));
        }
    });
    sleep(Duration::from_millis(200));
    for n in 1..6 {
        assert_eq!(1, expires_for_priming(1));
        {
            let c = EXPIRES_FOR_PRIMING.lock().unwrap();
            assert_eq!(n, c.cache_hits().unwrap());
            assert_eq!(0, c.cache_misses().unwrap());
            println!(
                "primed cache hits: {}, misses: {}",
                c.cache_hits().unwrap(),
                c.cache_misses().unwrap()
            );
        }
        sleep(Duration::from_millis(500));
    }
    h.join().unwrap();
    println!("now wait for expiration");
    sleep(Duration::from_millis(1000));
    assert_eq!(1, expires_for_priming(1));
    {
        let c = EXPIRES_FOR_PRIMING.lock().unwrap();
        assert_eq!(5, c.cache_hits().unwrap());
        assert_eq!(1, c.cache_misses().unwrap());
        println!(
            "primed cache hits: {}, misses: {}",
            c.cache_hits().unwrap(),
            c.cache_misses().unwrap()
        );
    }

    println!("\ndone!");
}
