//! Fibonacci Example

#[doc = "Kash static for the [`fib`] function."]
static FIB: ::kash::once_cell::sync::Lazy<::kash::moka::sync::Cache<u32, u32>> =
    ::kash::once_cell::sync::Lazy::new(|| {
        ::kash::moka::sync::Cache::builder()
            .eviction_policy(::kash::moka::policy::EvictionPolicy::tiny_lfu())
            .build()
    });
#[doc = "Origin of the function [`fib_no_cache`]."]
fn fib_no_cache(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib(n - 1) + fib(n - 2)
}
#[doc = "Primes the function [`fib`]."]
#[allow(dead_code)]
fn fib_prime_cache(n: u32) -> u32 {
    let kash_key = n;
    let kash_cache = FIB.clone();
    let kash_result = fib_no_cache(n);
    kash_cache.insert(kash_key, kash_result);
    kash_result
}
#[doc = "Caches the function [`fib`]."]
fn fib(n: u32) -> u32 {
    let kash_key = n;
    let kash_cache = FIB.clone();
    if let Some(kash_result) = kash_cache.get(&kash_key) {
        return kash_result.to_owned();
    }
    let kash_result = fib_no_cache(n);
    kash_cache.insert(kash_key, kash_result);
    kash_result
}

/// Fibonacci Example
pub fn main() {
    fib(1000);

    for i in 0..1000 {
        fib(i);
    }
}
