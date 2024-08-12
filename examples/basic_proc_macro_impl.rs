use kash::proc_macro::kash;
use std::thread::sleep;
use std::time::{Duration, Instant};

struct Example;

impl Example {
    #[kash(size = 50, in_impl)]
    pub fn slow_fn(n: u32) -> String {
        if n == 0 {
            return "done".to_string();
        }
        sleep(Duration::new(1, 0));
        Self::slow_fn(n - 1)
    }
}

pub fn main() {
    println!("[kash] Initial run...");
    let now = Instant::now();
    let _ = Example::slow_fn(10);
    println!("[kash] Elapsed: {}\n", now.elapsed().as_secs());

    println!("[kash] Kash run...");
    let now = Instant::now();
    let _ = Example::slow_fn(10);
    println!("[kash] Elapsed: {}\n", now.elapsed().as_secs());

    // Inspect the cache
    {
        use kash::Kash; // must be in scope to access cache

        println!("[kash] ** Cache info **");
        let cache = Example::slow_fn_get_cache_ident().lock().unwrap();
        assert_eq!(cache.cache_hits().unwrap(), 1);
        println!("[kash] hits=1 -> {:?}", cache.cache_hits().unwrap() == 1);
        assert_eq!(cache.cache_misses().unwrap(), 11);
        println!(
            "[kash] misses=11 -> {:?}",
            cache.cache_misses().unwrap() == 11
        );
        // make sure the cache-lock is dropped
    }

    println!("done!");
}
