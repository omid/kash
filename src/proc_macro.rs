#![cfg_attr(docsrs, doc(cfg(feature = "proc_macro")))]

/*!
Procedural macros for defining functions that wrap a static-ref cache object.

```rust,no_run
use std::thread::sleep;
use std::time::Duration;
use kash::proc_macro::kash;

/// Use an lru cache with size 100 and a `(String, String)` cache key
#[kash(size=100)]
fn keyed(a: String, b: String) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}
# pub fn main() { }
```

----

```rust,no_run
use std::thread::sleep;
use std::time::Duration;
use kash::proc_macro::kash;

/// Use a timed-lru cache with size 1, a TTL of 60s,
/// and a `(usize, usize)` cache key
#[kash(size=1, time=60)]
fn keyed(a: usize, b: usize) -> usize {
    let total = a + b;
    sleep(Duration::new(total as u64, 0));
    total
}
pub fn main() {
    keyed(1, 2);  // Not kash, will sleep (1+2)s

    keyed(1, 2);  // Kash, no sleep

    sleep(Duration::new(60, 0));  // Sleep for the TTL

    keyed(1, 2);  // 60s TTL has passed so the kash
                  // value has expired, will sleep (1+2)s

    keyed(1, 2);  // Kash, no sleep

    keyed(2, 1);  // New args, not kash, will sleep (2+1)s

    keyed(1, 2);  // Was evicted because of lru size of 1,
                  // will sleep (1+2)s
}
```

----

```rust,no_run
use std::thread::sleep;
use std::time::Duration;
use kash::proc_macro::kash;

/// Use a timed cache with a TTL of 60s
/// that refreshes the entry TTL on cache hit,
/// and a `(String, String)` cache key
#[kash(time=60, time_refresh=true)]
fn keyed(a: String, b: String) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}
# pub fn main() { }
```

----

```rust,no_run
use kash::proc_macro::kash;

# fn do_something_fallible() -> std::result::Result<(), ()> {
#     Ok(())
# }

/// Cache a fallible function. Only `Ok` results are kash.
#[kash(size=1, result = true)]
fn keyed(a: String) -> Result<usize, ()> {
    do_something_fallible()?;
    Ok(a.len())
}
# pub fn main() { }
```

----

```rust,no_run
use kash::proc_macro::kash;

/// Cache an optional function. Only `Some` results are kash.
#[kash(size=1, option = true)]
fn keyed(a: String) -> Option<usize> {
    if a == "a" {
        Some(a.len())
    } else {
        None
    }
}
# pub fn main() { }
```

----

```rust,no_run
use kash::proc_macro::kash;

/// Cache an optional function. Only `Some` results are kash.
/// When called concurrently, duplicate argument-calls will be
/// synchronized so as to only run once - the remaining concurrent
/// calls return a kash value.
#[kash(size=1, option = true, sync_writes = true)]
fn keyed(a: String) -> Option<usize> {
    if a == "a" {
        Some(a.len())
    } else {
        None
    }
}
# pub fn main() { }
```

----

```rust,no_run
use kash::proc_macro::kash;
use kash::Return;

/// Get a `kash::Return` value that indicates
/// whether the value returned came from the cache:
/// `kash::Return.was_cached`.
/// Use an LRU cache and a `String` cache key.
#[kash(size=1, wrap_return = true)]
fn calculate(a: String) -> Return<String> {
    Return::new(a)
}
pub fn main() {
    let r = calculate("a".to_string());
    assert!(!r.was_cached);
    let r = calculate("a".to_string());
    assert!(r.was_cached);
    // Return<String> derefs to String
    assert_eq!(r.to_uppercase(), "A");
}
```

----

```rust,no_run
use kash::proc_macro::kash;
use kash::Return;

# fn do_something_fallible() -> std::result::Result<(), ()> {
#     Ok(())
# }

/// Same as the previous, but returning a Result
#[kash(size=1, result = true, wrap_return = true)]
fn calculate(a: String) -> Result<Return<usize>, ()> {
    do_something_fallible()?;
    Ok(Return::new(a.len()))
}
pub fn main() {
    match calculate("a".to_string()) {
        Err(e) => eprintln!("error: {:?}", e),
        Ok(r) => {
            println!("value: {:?}, was cached: {}", *r, r.was_cached);
            // value: "a", was cached: true
        }
    }
}
```

----

```rust,no_run
use kash::proc_macro::kash;
use kash::Return;

/// Same as the previous, but returning an Option
#[kash(size=1, option = true, wrap_return = true)]
fn calculate(a: String) -> Option<Return<usize>> {
    if a == "a" {
        Some(Return::new(a.len()))
    } else {
        None
    }
}
pub fn main() {
    if let Some(a) = calculate("a".to_string()) {
        println!("value: {:?}, was cached: {}", *a, a.was_cached);
        // value: "a", was cached: true
    }
}
```

----

```rust,no_run
use std::thread::sleep;
use std::time::Duration;
use kash::proc_macro::kash;
use kash::SizedCache;

/// Use an explicit cache-type with a custom creation block and custom cache-key generating block
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
# pub fn main() { }
```

----

```rust
use std::thread::sleep;
use std::time::Duration;
use kash::proc_macro::kash;

/// Use a timed cache with a TTL of 60s.
/// Run a background thread to continuously refresh a specific key.
#[kash(time = 60, key = "String", convert = r#"{ String::from(a) }"#)]
fn keyed(a: &str) -> usize {
    a.len()
}
pub fn main() {
    let _handler = std::thread::spawn(|| {
        loop {
            sleep(Duration::from_secs(50));
            // this method is generated by the `kash` macro
            keyed_prime_cache("a");
        }
    });
    // handler.join().unwrap();
}
```

----

```rust
use std::thread::sleep;
use std::time::Duration;
use kash::proc_macro::kash;

/// Run a background thread to continuously refresh every key of a cache
#[kash(key = "String", convert = r#"{ String::from(a) }"#)]
fn keyed(a: &str) -> usize {
    a.len()
}
pub fn main() {
    let _handler = std::thread::spawn(|| {
        loop {
            sleep(Duration::from_secs(60));
            let keys: Vec<String> = {
                // note the cache keys are a tuple of all function arguments, unless it's one value
                KEYED.lock().unwrap().get_store().keys().map(|k| k.clone()).collect()
            };
            for k in &keys {
                // this method is generated by the `kash` macro
                keyed_prime_cache(k);
            }
        }
    });
    // handler.join().unwrap();
}
```


*/

#[doc(inline)]
pub use kash_proc_macro::{io_kash, kash};

/// Used to wrap a function result so callers can see whether the result was cached.
#[derive(Clone)]
pub struct Return<T> {
    pub was_cached: bool,
    pub value: T,
}

impl<T> Return<T> {
    pub fn new(value: T) -> Self {
        Self {
            was_cached: false,
            value,
        }
    }
}

impl<T> std::ops::Deref for Return<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> std::ops::DerefMut for Return<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
