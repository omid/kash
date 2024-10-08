use async_std::task::sleep;
use kash::proc_macro::kash;
use std::time::Duration;

async fn sleep_secs(secs: u64) {
    sleep(Duration::from_secs(secs)).await;
}

/// should only sleep the first time it's called
#[kash]
async fn kash_sleep_secs(secs: u64) {
    sleep(Duration::from_secs(secs)).await;
}

/// should only cache the result for a second, and only when
/// the result is `Ok`
#[kash(ttl = "1", key = "bool", convert = r#"{ true }"#, result)]
async fn only_kash_a_second(
    s: String,
) -> std::result::Result<Vec<String>, &'static dyn std::error::Error> {
    Ok(vec![s])
}

#[async_std::main]
async fn main() {
    let a = only_kash_a_second("a".to_string()).await.unwrap();
    let b = only_kash_a_second("b".to_string()).await.unwrap();
    assert_eq!(a, b);
    sleep_secs(1).await;
    let b = only_kash_a_second("b".to_string()).await.unwrap();
    assert_ne!(a, b);

    println!("kash sleeping for 1 seconds");
    kash_sleep_secs(1).await;
    println!("kash sleeping for 1 seconds");
    kash_sleep_secs(1).await;

    println!("done!");
}
