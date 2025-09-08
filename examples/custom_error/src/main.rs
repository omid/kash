#![allow(clippy::arithmetic_side_effects, clippy::unwrap_used)]
use kash::kash;
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

#[derive(Clone)]
enum MyError {
    Err,
}

#[kash(result)]
fn slow_fn(n: u32) -> Result<String, MyError> {
    if n == 0 {
        return Err(MyError::Err);
    }
    sleep(Duration::new(1, 0));
    slow_fn(n - 1)
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

    println!("done!");
}
