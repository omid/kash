#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
use kash::kash;
use std::{
    sync::Arc,
    thread::sleep,
    time::{Duration, Instant},
};

enum MyError {
    Err,
}

#[kash]
fn slow_fn(n: u32) -> Result<String, Arc<MyError>> {
    if n == 0 {
        return Err(Arc::new(MyError::Err));
    }
    sleep(Duration::new(1, 0));
    slow_fn(n - 1)
}

#[kash(result)]
fn slow_fn_with_result_flag(n: u32) -> Result<String, MyError> {
    if n == 0 {
        return Err(MyError::Err);
    }
    sleep(Duration::new(1, 0));
    slow_fn_with_result_flag(n - 1)
}

pub fn main() {
    println!("Initial run...");
    let now = Instant::now();
    let _ = slow_fn(10);
    println!("Elapsed: {}\n", now.elapsed().as_secs());

    println!("Cached run...");
    let now = Instant::now();
    let _ = slow_fn(10);
    println!("Elapsed: {}\n", now.elapsed().as_secs());

    println!("Initial run...");
    let now = Instant::now();
    let _ = slow_fn_with_result_flag(10);
    println!("Elapsed: {}\n", now.elapsed().as_secs());

    println!("Cached run...");
    let now = Instant::now();
    let _ = slow_fn_with_result_flag(10);
    println!("Elapsed: {}\n", now.elapsed().as_secs());

    println!("done!");
}
