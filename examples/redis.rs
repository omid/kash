/*
Start a redis docker image if you don't already have it running locally:
    docker run --rm --name kash-redis-example -p 6379:6379 -d redis
Set the required env variable and run this example and run with required features:
    KASH_REDIS_CONNECTION_STRING=redis://127.0.0.1:6379 cargo run --example redis --features "redis_store"
Cleanup the redis docker container:
    docker rm -f kash-redis-example
 */

use kash::proc_macro::io_kash;
use kash::RedisCache;
use once_cell::sync::Lazy;
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
    time = 30,
    cache_prefix_block = r##"{ "cache-redis-example-1" }"##,
    map_error = r##"|e| ExampleError::RedisError(format!("{:?}", e))"##
)]
fn kash_sleep_secs(secs: u64) -> Result<(), ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(())
}

// If not `cache_prefix_block` is specified, then the function name
// is used to create a prefix for cache keys used by this function
#[io_kash(
    redis,
    time = 30,
    map_error = r##"|e| ExampleError::RedisError(format!("{:?}", e))"##
)]
fn kash_sleep_secs_example_2(secs: u64) -> Result<(), ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(())
}

struct Config {
    conn_str: String,
}
impl Config {
    fn load() -> Self {
        Self {
            conn_str: std::env::var("KASH_REDIS_CONNECTION_STRING").unwrap(),
        }
    }
}

static CONFIG: Lazy<Config> = Lazy::new(Config::load);

#[io_kash(
    map_error = r##"|e| ExampleError::RedisError(format!("{:?}", e))"##,
    ty = "kash::RedisCache<u64, String>",
    create = r##" {
        RedisCache::new("cache_redis_example_kash_sleep_secs_config", 1)
            .set_refresh(true)
            .set_connection_string(&CONFIG.conn_str)
            .build()
            .expect("error building example redis cache")
    } "##
)]
fn kash_sleep_secs_config(secs: u64) -> Result<String, ExampleError> {
    std::thread::sleep(Duration::from_secs(secs));
    Ok(secs.to_string())
}

#[tokio::main]
async fn main() {
    print!("1. first sync call with a 2 seconds sleep...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");
    print!("second sync call with a 2 seconds sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");

    use kash::IOKash;
    KASH_SLEEP_SECS.cache_remove(&2).unwrap();
    print!("third sync call with a 2 seconds sleep (slow, after cache-remove)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs(2).unwrap();
    println!("done");

    print!("2. first sync call with a 2 seconds sleep...");
    io::stdout().flush().unwrap();
    kash_sleep_secs_example_2(2).unwrap();
    println!("done");
    print!("second sync call with a 2 seconds sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs_example_2(2).unwrap();
    println!("done");

    kash_sleep_secs_config_prime_cache(2).unwrap();
    print!("3. first primed async call with a 2 seconds sleep (should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs_config(2).unwrap();
    println!("done");
    print!("second async call with a 2 seconds sleep (it should be fast)...");
    io::stdout().flush().unwrap();
    kash_sleep_secs_config(2).unwrap();
    println!("done");
}
