/*
run with required features:
    cargo run --example disk --features "disk_store"
 */

use kash::proc_macro::io_kash;
use std::io;
use std::io::Write;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum ExampleError {
    #[error("error with disk cache `{0}`")]
    DiskError(String),
}

// When the macro constructs your DiskCache instance, the default
// cache files will be stored under $system_cache_dir/kash_disk_cache/
#[io_kash(
    disk,
    time = 30,
    map_error = r##"|e| ExampleError::DiskError(format!("{:?}", e))"##
)]
fn kash_sleep_secs(secs: u64) -> Result<(), ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(())
}

fn main() {
    print!("1. first sync call with a 2 seconds sleep...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");
    print!("second sync call with a 2 seconds sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");

    use kash::IOKash;
    KASH_SLEEP_SECS.remove(&2).unwrap();
    print!("third sync call with a 2 seconds sleep (slow, after cache-remove)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");
}
