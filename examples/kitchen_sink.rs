use std::thread::sleep;
use std::time::Duration;

use kash_proc_macro::kash;

// kash shorthand, uses the default unbounded cache.
// Equivalent to specifying `FIB: UnboundCache<(u32), u32> = UnboundCache::new();`
#[kash]
fn fib(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib(n - 1) + fib(n - 2)
}

// Same as above, but preallocates some space.
// Note that the cache key type is a tuple of function argument types.
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
#[kash(size = "100", key = "String", convert = r#"format!("{a}{b}")"#)]
fn keyed(a: &str, b: &str) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}

pub fn main() {
    println!("\n ** default cache **");
    fib(3);
    fib(3);
    {
        // let cache = FIB.lock().unwrap();
        // println!("hits: {:?}", cache.cache_hits());
        // println!("misses: {:?}", cache.cache_misses());
        // make sure lock is dropped
    }
    fib(10);
    fib(10);

    println!("\n ** specific cache **");
    fib_specific(20);
    fib_specific(20);
    {
        // let cache = FIB_SPECIFIC.lock().unwrap();
        // println!("hits: {:?}", cache.cache_hits());
        // println!("misses: {:?}", cache.cache_misses());
        // make sure lock is dropped
    }
    fib_specific(20);
    fib_specific(20);

    println!("done!");
}
