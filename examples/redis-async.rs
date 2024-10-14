/*
Start a redis docker image if you don't already have it running locally:
    docker run --rm --name async-kash-redis-example -p 6379:6379 -d redis
Set the required env variable and run this example and run with required features:
    KASH_REDIS_CONNECTION_STRING=redis://127.0.0.1:6379 cargo run --example redis-async --features "async redis_store redis_tokio"
Cleanup the redis docker container:
    docker rm -f async-kash-redis-example
 */

use kash::io_kash;
use std::io;
use std::io::Write;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
enum ExampleError {
    #[error("error with redis cache `{0}`")]
    RedisError(String),
}

// When the macro constructs your RedisCache instance, the connection string
// will be pulled from the env var: `KASH_REDIS_CONNECTION_STRING`;
#[io_kash(
    redis,
    ttl = 30,
    cache_prefix_block = r##"{ "cache-redis-example-1" }"##,
    map_error = r##"|e| ExampleError::RedisError(format!("{:?}", e))"##
)]
async fn kash_sleep_secs(secs: u64) -> Result<(), ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(())
}

#[io_kash(
    map_error = r##"|e| ExampleError::RedisError(format!("{:?}", e))"##,
    redis
)]
async fn async_kash_sleep_secs(secs: u64) -> Result<String, ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(secs.to_string())
}

#[tokio::main]
async fn main() {
    print!("1. first sync call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).await.unwrap();
    println!("done");
    print!("second sync call with a 2-second sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).await.unwrap();
    println!("done");

    print!("2. first async call with a 2-second sleep...");
    io::stdout().flush().unwrap();
    async_kash_sleep_secs(2).await.unwrap();
    println!("done");
    print!("second async call with a 2-second sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    async_kash_sleep_secs(2).await.unwrap();
    println!("done");
}
