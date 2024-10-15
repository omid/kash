/*
run with required features:
    cargo run --example disk --features "disk_store"
 */

use kash::{io_kash, DiskCacheError};
use std::io;
use std::io::Write;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum ExampleError {
    #[error("error with disk cache `{0}`")]
    DiskError(String),
    #[error("error count `{0}`")]
    Count(i32),
}

impl From<DiskCacheError> for ExampleError {
    fn from(e: DiskCacheError) -> Self {
        ExampleError::DiskError(format!("{:?}", e))
    }
}

// When the macro constructs your DiskCache instance, the default
// cache files will be stored under $system_cache_dir/kash_disk_cache/
#[io_kash(disk, ttl = "30")]
fn kash_sleep_secs(secs: u64) -> Result<i32, ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(5)
}

#[io_kash(disk, ttl = "1", option)]
fn kash_disk_optional(n: u32) -> Result<Option<u32>, ExampleError> {
    std::thread::sleep(Duration::from_secs(2));
    if n < 5 {
        Ok(Some(n))
    } else {
        Err(ExampleError::Count(1))
    }
}

#[io_kash(disk, ttl = "1", result)]
fn kash_disk_result(n: u32) -> Result<u32, ExampleError> {
    std::thread::sleep(Duration::from_secs(2));
    if n < 5 {
        Ok(n)
    } else {
        Err(ExampleError::Count(1))
    }
}

fn main() {
    print!("1. first sync call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    assert_eq!(kash_disk_optional(1), Ok(Some(1)));
    println!("done");
    print!("second sync call with a 2-second sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    assert_eq!(kash_disk_optional(1), Ok(Some(1)));
    println!("done");
    io::stdout().flush().unwrap();

    print!("1. first sync call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    assert_eq!(kash_disk_optional(5), Err(ExampleError::Count(1)));
    println!("done");
    print!("second sync call with a 1-second sleep (it should be still slow)...");
    io::stdout().flush().unwrap();
    assert_eq!(kash_disk_optional(6), Err(ExampleError::Count(1)));
    println!("done");
    io::stdout().flush().unwrap();

    //////////////////////////////////////
    print!("1. first sync call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    assert_eq!(kash_disk_result(1), Ok(1));
    println!("done");
    print!("second sync call with a 2-second sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    assert_eq!(kash_disk_result(1), Ok(1));
    println!("done");
    io::stdout().flush().unwrap();

    print!("1. first sync call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    assert_eq!(kash_disk_result(5), Err(ExampleError::Count(1)));
    println!("done");
    print!("second sync call with a 1-second sleep (it should be still slow)...");
    io::stdout().flush().unwrap();
    assert_eq!(kash_disk_result(6), Err(ExampleError::Count(1)));
    println!("done");
    io::stdout().flush().unwrap();

    //////////////////////////////////////
    print!("1. first sync call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");
    print!("second sync call with a 2-second sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");

    use kash::IOKash;
    KASH_SLEEP_SECS.remove(&2).unwrap();
    print!("third sync call with a 2-second sleep (slow, after cache-remove)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");
}
