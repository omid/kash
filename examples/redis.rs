#![allow(clippy::unwrap_used)]
/*
Start a redis docker image if you don't already have it running locally:
    docker run --rm --name kash-redis-example -p 6379:6379 -d redis
Set the required env variable and run this example and run with required features:
    KASH_REDIS_CONNECTION_STRING=redis://127.0.0.1:6379 cargo run --example redis --features "redis_store"
Cleanup the redis docker container:
    docker rm -f kash-redis-example
 */

use kash::{RedisCacheError, kash};
use std::io;
use std::io::Write;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum ExampleError {
    #[error("error with redis cache `{0}`")]
    RedisError(String),
}

impl From<RedisCacheError> for ExampleError {
    fn from(e: RedisCacheError) -> Self {
        ExampleError::RedisError(format!("{e:?}"))
    }
}

// When the macro constructs your RedisCache instance, the connection string
// will be pulled from the env var: `KASH_REDIS_CONNECTION_STRING`;
#[allow(clippy::unnecessary_wraps)]
#[kash(redis(prefix_block = r#"{ "cache-redis-example-2:" }"#), ttl = "30")]
fn kash_sleep_secs(secs: u64) -> Result<(), ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(())
}

// If not `prefix_block` is specified, then the function name
// is used to create a prefix for cache keys used by this function
#[allow(clippy::unnecessary_wraps)]
#[kash(redis, ttl = "30")]
fn kash_sleep_secs_example_2(secs: u64) -> Result<(), ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(())
}

#[tokio::main]
async fn main() {
    use kash::IOKash;

    print!("1. first sync call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");
    print!("second sync call with a 2-second sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");

    KASH_SLEEP_SECS.remove(&2).unwrap();
    print!("third sync call with a 2-second sleep (slow, after cache-remove)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");

    print!("2. first sync call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    kash_sleep_secs_example_2(2).unwrap();
    println!("done");
    print!("second sync call with a 2-second sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs_example_2(2).unwrap();
    println!("done");
}
