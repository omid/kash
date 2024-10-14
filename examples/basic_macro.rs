use kash::kash;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[kash(size = "50")]
fn slow_fn(n: u32) -> String {
    if n == 0 {
        return "done".to_string();
    }
    sleep(Duration::new(1, 0));
    slow_fn(n - 1)
}

#[kash(size = "50")]
pub fn slow_fn_with_ref_mut_self(n: u32) -> String {
    if n == 0 {
        return "done".to_string();
    }
    sleep(Duration::new(1, 0));
    slow_fn_with_ref_mut_self(n - 1)
}

#[kash(size = "50")]
pub fn slow_fn_with_self(n: u32) -> String {
    if n == 0 {
        return "done".to_string();
    }
    sleep(Duration::new(1, 0));
    slow_fn_with_self(n - 1)
}

#[kash(size = "50")]
pub fn slow_fn_with_ref_self(n: u32) -> String {
    if n == 0 {
        return "done".to_string();
    }
    sleep(Duration::new(1, 0));
    slow_fn_with_ref_self(n - 1)
}

#[allow(unused_mut)]
#[kash(size = "50")]
pub fn slow_fn_with_mut_self(n: u32) -> String {
    if n == 0 {
        return "done".to_string();
    }
    sleep(Duration::new(1, 0));
    slow_fn_with_mut_self(n - 1)
}

#[allow(clippy::needless_lifetimes)]
#[kash(size = "50")]
pub fn slow_fn_with_lifetime<'a>(n: &'a i32) -> String {
    if *n == 0 {
        return "done".to_string();
    }
    sleep(Duration::new(1, 0));
    slow_fn_with_lifetime(&(n - 1))
}

pub fn main() {
    println!("[kash] Initial run...");
    let now = Instant::now();
    let _ = slow_fn(10);
    println!("[kash] Elapsed: {}\n", now.elapsed().as_secs());

    println!("[kash] Kash run...");
    let now = Instant::now();
    let _ = slow_fn(10);
    println!("[kash] Elapsed: {}\n", now.elapsed().as_secs());

    // Inspect the cache
    // {
    //     println!("[kash] ** Cache info **");
    //     let cache = SLOW_FN.clone();
    //     assert_eq!(cache.cache_hits().unwrap(), 1);
    //     println!("[kash] hits=1 -> {:?}", cache.cache_hits().unwrap() == 1);
    //     assert_eq!(cache.cache_misses().unwrap(), 11);
    //     println!(
    //         "[kash] misses=11 -> {:?}",
    //         cache.cache_misses().unwrap() == 11
    //     );
    //     // make sure the cache-lock is dropped
    // }

    println!("done!");
}
