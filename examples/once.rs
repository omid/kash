use kash::kash;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

// If you forcefully want to run a function only once
#[kash(sync_writes, key(ty = "bool", expr = "{true}"))]
fn slow_fn(str: &str) -> String {
    sleep(Duration::new(2, 0));
    let res = str.to_string();
    println!("{res}");
    res
}

pub fn main() {
    println!("Initial run...");
    let now = Instant::now();
    let _ = slow_fn("10");
    println!("Elapsed: {}\n", now.elapsed().as_secs());

    println!("Cached run...");
    let now = Instant::now();
    let _ = slow_fn("11");
    println!("Elapsed: {}\n", now.elapsed().as_secs());

    println!("done!");
}
