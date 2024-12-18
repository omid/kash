use kash::kash;
use std::time::Duration;
use tokio::time::sleep;

async fn sleep_secs(secs: u64) {
    sleep(Duration::from_secs(secs)).await;
}

#[kash]
async fn kash_sleep_secs(secs: u64) {
    sleep(Duration::from_secs(secs)).await;
}

#[kash]
fn kash_sleep_secs2(secs: u64) {
    println!("{secs}");
}

#[tokio::main]
async fn main() {
    println!("sleeping for 4 seconds");
    sleep_secs(4).await;
    println!("first cached sleeping for 4 seconds");
    kash_sleep_secs(4).await;
    println!("second cached sleeping for 4 seconds");
    kash_sleep_secs(4).await;

    println!("done!");
}
