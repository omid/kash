#![allow(
    clippy::arithmetic_side_effects,
    clippy::unwrap_used,
    clippy::unnecessary_wraps
)]
use kash::kash;
use std::thread::{sleep, spawn};
use std::time::Duration;

// `kash` shorthand, uses the default unbounded cache.
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

// Same as above, but pre-allocates some space.
#[kash(size = "50")]
fn fib_specific(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib_specific(n - 1) + fib_specific(n - 2)
}

// Specify a specific cache type
// Note that the cache key type is a tuple of function argument types.
#[kash(size = "100")]
fn slow(a: u32, b: u32) -> u32 {
    sleep(Duration::new(2, 0));
    a * b
}

// Specify a specific cache type and an explicit key expression
// Note that the cache key type is a `String` created from the borrow arguments
#[kash(
    size = "100",
    key(ty = "String", expr = r#"{ format!("{}{}", a, b) }"#)
)]
fn keyed(a: &str, b: &str) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}

#[kash(key(ty = "String", expr = r#"{ format!("{}{}", a, b) }"#))]
fn keyed_key(a: &str, b: &str) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}

// handle results, don't cache errors
#[kash(result)]
fn slow_result(a: u32, b: u32) -> Result<u32, ()> {
    sleep(Duration::new(2, 0));
    Ok(a * b)
}

// A simple cache that expires after a second. We'll keep the
// value fresh by priming it in a separate thread.
#[kash(ttl = "1")]
fn expires_for_priming(a: i32) -> i32 {
    a
}

pub fn main() {
    println!("\n ** default cache with default name **");
    fib(3);
    fib(3);
    // {
    //     let cache = FIB.clone();
    //     println!("hits: {:?}", cache.cache_hits());
    //     assert_eq!(cache.cache_hits().unwrap(), 2);
    //     println!("misses: {:?}", cache.cache_misses());
    //     assert_eq!(cache.cache_misses(), Some(4));
    //     // make sure lock is dropped
    // }
    fib(10);
    fib(10);

    println!("\n ** default cache with explicit name **");
    fib_2(3);
    fib_2(3);
    // {
    //     let cache = FLIB.clone();
    //     println!("hits: {:?}", cache.cache_hits());
    //     assert_eq!(cache.cache_hits().unwrap(), 1);
    //     println!("misses: {:?}", cache.cache_misses());
    //     assert_eq!(cache.cache_misses(), Some(1));
    //     // make sure lock is dropped
    // }

    println!("\n ** specific cache **");
    fib_specific(20);
    fib_specific(20);
    // {
    //     let cache = FIB_SPECIFIC.clone();
    //     println!("hits: {:?}", cache.cache_hits());
    //     assert_eq!(cache.cache_hits().unwrap(), 19);
    //     println!("misses: {:?}", cache.cache_misses());
    //     assert_eq!(cache.cache_misses(), Some(21));
    //     // make sure lock is dropped
    // }
    fib_specific(20);
    fib_specific(20);

    println!("\n ** slow func **");
    println!(" - first run `slow(10)`");
    slow(10, 10);
    println!(" - second run `slow(10)`");
    slow(10, 10);
    // {
    //     let cache = SLOW.clone();
    //     println!("hits: {:?}", cache.cache_hits());
    //     assert_eq!(cache.cache_hits().unwrap(), 1);
    //     println!("misses: {:?}", cache.cache_misses());
    //     assert_eq!(cache.cache_misses().unwrap(), 1);
    //     // make sure the cache-lock is dropped
    // }

    println!("\n ** slow result func **");
    println!(" - first run `slow_result(10)`");
    let _ = slow_result(10, 10);
    println!(" - second run `slow_result(10)`");
    let _ = slow_result(10, 10);
    // {
    //     let cache = SLOW_RESULT.clone();
    //     println!("hits: {:?}", cache.cache_hits());
    //     assert_eq!(cache.cache_hits().unwrap(), 1);
    //     println!("misses: {:?}", cache.cache_misses());
    //     assert_eq!(cache.cache_misses(), Some(1));
    //     // make sure the cache-lock is dropped
    // }

    println!("\n ** refresh by priming **");
    let h = spawn(|| {
        for _ in 1..6 {
            expires_for_priming_prime_cache(1);
            sleep(Duration::from_millis(500));
        }
    });
    sleep(Duration::from_millis(200));
    for _n in 1..6 {
        assert_eq!(1, expires_for_priming(1));
        // {
        //     let c = EXPIRES_FOR_PRIMING.clone();
        //     assert_eq!(n, c.cache_hits().unwrap());
        //     assert_eq!(0, c.cache_misses().unwrap());
        //     println!(
        //         "primed cache hits: {}, misses: {}",
        //         c.cache_hits().unwrap(),
        //         c.cache_misses().unwrap()
        //     );
        // }
        sleep(Duration::from_millis(500));
    }
    h.join().unwrap();
    println!("now wait for expiration");
    sleep(Duration::from_millis(1000));
    assert_eq!(1, expires_for_priming(1));
    // {
    //     let c = EXPIRES_FOR_PRIMING.clone();
    //     assert_eq!(5, c.cache_hits().unwrap());
    //     assert_eq!(1, c.cache_misses().unwrap());
    //     println!(
    //         "primed cache hits: {}, misses: {}",
    //         c.cache_hits().unwrap(),
    //         c.cache_misses().unwrap()
    //     );
    // }

    println!("\ndone!");
}
