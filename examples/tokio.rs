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

#[kash(result, wrap_return)]
async fn kash_was_cached(count: u32) -> Result<kash::Return<String>, ()> {
    Ok(kash::Return::new(
        (0..count).map(|_| "a").collect::<Vec<_>>().join(""),
    ))
}

#[tokio::main]
async fn main() {
    println!("sleeping for 4 seconds");
    sleep_secs(4).await;
    println!("first kash sleeping for 4 seconds");
    kash_sleep_secs(4).await;
    println!("second kash sleeping for 4 seconds");
    kash_sleep_secs(4).await;

    let a = kash_was_cached(4).await.unwrap();
    assert_eq!(a.to_uppercase(), "AAAA");
    assert!(!a.was_cached);
    let a = kash_was_cached(4).await.unwrap();
    assert!(a.was_cached);

    println!("done!");
}
