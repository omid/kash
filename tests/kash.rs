use kash::kash;
use std::sync::Arc;
use std::thread::{self, sleep};
use std::time::Duration;

#[kash]
fn fib0(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib0(n - 1) + fib0(n - 2)
}

#[test]
fn test_unbound_cache() {
    fib0(20);
    {
        FIB0.run_pending_tasks();
        let cache_size = FIB0.entry_count();
        assert_eq!(21, cache_size);
    }
}

#[kash(size = "3")]
fn fib1(n: u32) -> u32 {
    if n == 0 || n == 1 {
        return n;
    }
    fib1(n - 1) + fib1(n - 2)
}

#[test]
fn test_sized_cache() {
    fib1(20);
    {
        FIB1.run_pending_tasks();
        let cache_size = FIB1.entry_count();
        assert_eq!(3, cache_size);
        let items = FIB1.iter().collect::<Vec<_>>();
        assert_eq!(3, items.len());
    }
}

// #[kash(size = "5", ttl = "2")]
// fn timed(n: u32) -> u32 {
//     sleep(Duration::new(3, 0));
//     n
// }

// #[test]
// fn test_timed_cache() {
//     timed(1);
//     timed(1);
// let cache = TIMED.lock().unwrap();
// assert_eq!(1, cache.cache_misses().unwrap());
// assert_eq!(1, cache.cache_hits().unwrap());
//     sleep(Duration::new(3, 0));
//     timed(1);
// let cache = TIMED.lock().unwrap();
// assert_eq!(2, cache.cache_misses().unwrap());
// assert_eq!(1, cache.cache_hits().unwrap());
//     timed(1);
//     sleep(Duration::new(1, 0));
//     timed(1);
// let cache = TIMED.lock().unwrap();
// assert_eq!(3, cache.cache_misses().unwrap());
// assert_eq!(2, cache.cache_hits().unwrap());
// }

// #[kash(size = "3", ttl = "2")]
// fn timefac(n: u32) -> u32 {
//     sleep(Duration::new(1, 0));
//     if n > 1 {
//         n * timefac(n - 1)
//     } else {
//         n
//     }
// }

// #[test]
// fn test_timed_sized_cache() {
//     timefac(1);
//     timefac(1);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(1, cache.cache_misses().unwrap());
//         assert_eq!(1, cache.cache_hits().unwrap());
//     sleep(Duration::new(3, 0));
//     timefac(1);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(2, cache.cache_misses().unwrap());
//         assert_eq!(1, cache.cache_hits().unwrap());
//     timefac(1);
//     sleep(Duration::new(1, 0));
//     timefac(1);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(3, cache.cache_misses().unwrap());
//         assert_eq!(2, cache.cache_hits().unwrap());
//         let mut cache = TIMEFAC.lock().unwrap();
//         assert_eq!(1, cache.set_ttl(6).unwrap());
//     timefac(2);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(4, cache.cache_misses().unwrap());
//         assert_eq!(3, cache.cache_hits().unwrap());
//     timefac(3);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(5, cache.cache_misses().unwrap());
//         assert_eq!(4, cache.cache_hits().unwrap());
//     timefac(3);
//     timefac(2);
//     timefac(1);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(5, cache.cache_misses().unwrap());
//         assert_eq!(7, cache.cache_hits().unwrap());
//     timefac(4);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(6, cache.cache_misses().unwrap());
//         assert_eq!(8, cache.cache_hits().unwrap());
//     timefac(6);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(8, cache.cache_misses().unwrap());
//         assert_eq!(9, cache.cache_hits().unwrap());
//     timefac(1);
//         let cache = TIMEFAC.lock().unwrap();
//         assert_eq!(9, cache.cache_misses().unwrap());
//         assert_eq!(9, cache.cache_hits().unwrap());
//         assert_eq!(3, cache.cache_size());
// }

#[kash(size = "1")]
fn string_1(a: String, b: String) -> String {
    a + &b
}

#[test]
fn test_string_cache() {
    string_1("a".into(), "b".into());
    STRING_1.run_pending_tasks();
    assert_eq!(1, STRING_1.entry_count());
}

// #[kash(size = "5", ttl = "2")]
// fn timed_2(n: u32) -> u32 {
//     sleep(Duration::new(3, 0));
//     n
// }

// #[test]
// fn test_timed_cache_key() {
//     timed_2(1);
//     timed_2(1);
//         let cache = TIMED_2.lock().unwrap();
//         assert_eq!(1, cache.cache_misses().unwrap());
//         assert_eq!(1, cache.cache_hits().unwrap());
//     sleep(Duration::new(3, 0));
//     timed_2(1);
//         let cache = TIMED_2.lock().unwrap();
//         assert_eq!(2, cache.cache_misses().unwrap());
//         assert_eq!(1, cache.cache_hits().unwrap());
// }

#[kash(size = "2", key(ty = "String", expr = r#"format!("{a}{b}")"#))]
fn sized_key(a: &str, b: &str) -> usize {
    let size = a.len() + b.len();
    sleep(Duration::new(size as u64, 0));
    size
}

#[test]
fn test_sized_cache_key() {
    sized_key("a", "1");
    sized_key("a", "1");
    // assert_eq!(1, cache.cache_misses().unwrap());
    // assert_eq!(1, cache.cache_hits().unwrap());
    SIZED_KEY.run_pending_tasks();
    assert_eq!(1, SIZED_KEY.entry_count());
    sized_key("a", "2");
    // assert_eq!(2, cache.cache_hits().unwrap());
    SIZED_KEY.run_pending_tasks();
    assert_eq!(2, SIZED_KEY.entry_count());

    let (keys, values): (Vec<_>, Vec<_>) = SIZED_KEY.into_iter().unzip();
    assert!(keys.contains(&Arc::new("a1".to_string())));
    assert!(keys.contains(&Arc::new("a2".to_string())));
    assert_eq!(vec![2, 2], values);

    sized_key("a", "3");
    SIZED_KEY.run_pending_tasks();
    assert_eq!(2, SIZED_KEY.entry_count());

    let (keys, values): (Vec<_>, Vec<_>) = SIZED_KEY.into_iter().unzip();
    assert!(keys.contains(&Arc::new("a3".to_string())));
    assert!(keys.contains(&Arc::new("a2".to_string())));
    assert_eq!(vec![2, 2], values);

    sized_key("a", "4");
    sized_key("a", "5");
    SIZED_KEY.run_pending_tasks();
    assert_eq!(2, SIZED_KEY.entry_count());

    let (keys, values): (Vec<_>, Vec<_>) = SIZED_KEY.into_iter().unzip();
    assert!(keys.contains(&Arc::new("a4".to_string())));
    assert!(keys.contains(&Arc::new("a5".to_string())));
    assert_eq!(vec![2, 2], values);

    sized_key("a", "67");
    sized_key("a", "8");
    SIZED_KEY.run_pending_tasks();
    assert_eq!(2, SIZED_KEY.entry_count());

    let (keys, values): (Vec<_>, Vec<_>) = SIZED_KEY.into_iter().unzip();
    assert!(keys.contains(&Arc::new("a67".to_string())));
    assert!(keys.contains(&Arc::new("a8".to_string())));
    assert!(values.contains(&2));
    assert!(values.contains(&3));
}

#[kash(result)]
fn test_result_key(n: u32) -> Result<u32, ()> {
    if n < 5 {
        Ok(n)
    } else {
        Err(())
    }
}

#[test]
fn cache_result_key() {
    assert!(test_result_key(2).is_ok());
    assert!(test_result_key(4).is_ok());
    assert!(test_result_key(6).is_err());
    assert!(test_result_key(6).is_err());
    assert!(test_result_key(2).is_ok());
    assert!(test_result_key(4).is_ok());
    TEST_RESULT_KEY.run_pending_tasks();
    assert_eq!(2, TEST_RESULT_KEY.entry_count());
}

#[kash(result)]
fn test_result_no_default(n: u32) -> Result<u32, ()> {
    if n < 5 {
        Ok(n)
    } else {
        Err(())
    }
}

#[test]
fn cache_result_no_default() {
    assert!(test_result_no_default(2).is_ok());
    assert!(test_result_no_default(4).is_ok());
    assert!(test_result_no_default(6).is_err());
    assert!(test_result_no_default(6).is_err());
    assert!(test_result_no_default(2).is_ok());
    assert!(test_result_no_default(4).is_ok());
    TEST_RESULT_NO_DEFAULT.run_pending_tasks();
    assert_eq!(2, TEST_RESULT_NO_DEFAULT.entry_count());
}

#[kash(size = "2", key(ty = "String", expr = r#"format!("{a}/{b}")"#))]
fn slow_small_cache(a: &str, b: &str) -> String {
    sleep(Duration::new(1, 0));
    format!("{a}:{b}")
}

#[test]
/// This is a regression test to confirm that racing cache sets on a `SizedCache`
/// do not cause duplicates to exist in the internal `order`. See issue #7
fn test_racing_duplicate_keys_do_not_duplicate_sized_cache_ordering() {
    let a = thread::spawn(|| slow_small_cache("a", "b"));
    sleep(Duration::new(0, 500_000));
    let b = thread::spawn(|| slow_small_cache("a", "b"));
    a.join().unwrap();
    b.join().unwrap();
    // At this point, the cache should have a size of one since the keys are the same
    // and the internal `order` list should also have one item.
    // Since the method's cache has a capacity of 2, caching two more unique keys should
    // force the full eviction of the original values.
    slow_small_cache("c", "d");
    slow_small_cache("e", "f");
    slow_small_cache("g", "h");
}

// NoClone is not cloneable. So this also tests that the Result type
// itself does not have to be cloneable, just the type for the Ok
// value.
// Vec has Clone, but not Copy, to make sure Copy isn't required.
struct NoClone {}

#[kash(result)]
fn proc_kash_result(n: u32) -> Result<Vec<u32>, NoClone> {
    if n < 5 {
        Ok(vec![n])
    } else {
        Err(NoClone {})
    }
}

#[test]
fn test_proc_kash_result() {
    assert!(proc_kash_result(2).is_ok());
    assert!(proc_kash_result(4).is_ok());
    assert!(proc_kash_result(6).is_err());
    assert!(proc_kash_result(6).is_err());
    assert!(proc_kash_result(2).is_ok());
    assert!(proc_kash_result(4).is_ok());
    PROC_KASH_RESULT.run_pending_tasks();
    assert_eq!(2, PROC_KASH_RESULT.entry_count());
}

#[kash(option)]
fn proc_kash_option(n: u32) -> Option<Vec<u32>> {
    if n < 5 {
        Some(vec![n])
    } else {
        None
    }
}

#[test]
fn test_proc_kash_option() {
    assert!(proc_kash_option(2).is_some());
    assert!(proc_kash_option(4).is_some());
    assert!(proc_kash_option(1).is_some());
    assert!(proc_kash_option(6).is_none());
    assert!(proc_kash_option(6).is_none());
    assert!(proc_kash_option(2).is_some());
    assert!(proc_kash_option(1).is_some());
    assert!(proc_kash_option(4).is_some());
    PROC_KASH_OPTION.run_pending_tasks();
    assert_eq!(3, PROC_KASH_OPTION.entry_count());
}

#[kash]
fn test_result_missing_result_arm(n: u32) -> Result<u32, ()> {
    Ok(n)
}

#[kash]
fn test_result_key_missing_result_arm(n: u32) -> Result<u32, ()> {
    Ok(n)
}

// #[kash(size = "1", ttl = "1")]
// fn proc_timed_sized_sleeper(n: u64) -> u64 {
//     sleep(Duration::new(1, 0));
//     n
// }

// #[test]
// fn test_proc_timed_sized_cache() {
//     proc_timed_sized_sleeper(1);
//     proc_timed_sized_sleeper(1);
//         let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
//         assert_eq!(1, cache.cache_misses().unwrap());
//         assert_eq!(1, cache.cache_hits().unwrap());
//     // sleep to expire the one entry
//     sleep(Duration::new(1, 0));
//     proc_timed_sized_sleeper(1);
//         let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
//         assert_eq!(2, cache.cache_misses().unwrap());
//         assert_eq!(1, cache.cache_hits().unwrap());
//         assert_eq!(cache.key_order().collect::<Vec<_>>(), vec![&1]);
//     // sleep to expire the one entry
//     sleep(Duration::new(1, 0));
//         let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
//         assert!(cache.key_order().next().is_none());
//     proc_timed_sized_sleeper(1);
//     proc_timed_sized_sleeper(1);
//         let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
//         assert_eq!(3, cache.cache_misses().unwrap());
//         assert_eq!(2, cache.cache_hits().unwrap());
//         assert_eq!(cache.key_order().collect::<Vec<_>>(), vec![&1]);
//     // lru size is 1, so this new thing evicts the existing key
//     proc_timed_sized_sleeper(2);
//         let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
//         assert_eq!(4, cache.cache_misses().unwrap());
//         assert_eq!(2, cache.cache_hits().unwrap());
//         assert_eq!(cache.key_order().collect::<Vec<_>>(), vec![&2]);
// }

// #[kash(size = "2")]
// fn kash_smartstring(s: smartstring::alias::String) -> smartstring::alias::String {
//     if s == "very stringy" {
//         smartstring::alias::String::from("equal")
//     } else {
//         smartstring::alias::String::from("not equal")
//     }
// }

// #[test]
// fn test_kash_smartstring() {
//     let mut string = smartstring::alias::String::new();
//     string.push_str("very stringy");
//     assert_eq!("equal", kash_smartstring(string.clone()));
//         let cache = KASH_SMARTSTRING.lock().unwrap();
//         assert_eq!(cache.cache_hits(), Some(0));
//         assert_eq!(cache.cache_misses(), Some(1));

//     assert_eq!("equal", kash_smartstring(string.clone()));
//         let cache = KASH_SMARTSTRING.lock().unwrap();
//         assert_eq!(cache.cache_hits(), Some(1));
//         assert_eq!(cache.cache_misses(), Some(1));

//     let string = smartstring::alias::String::from("also stringy");
//     assert_eq!("not equal", kash_smartstring(string));
//         let cache = KASH_SMARTSTRING.lock().unwrap();
//         assert_eq!(cache.cache_hits(), Some(1));
//         assert_eq!(cache.cache_misses(), Some(2));
// }

#[allow(unused_mut)]
#[kash]
fn mutable_args(mut a: i32, mut b: i32) -> (i32, i32) {
    a += 1;
    b += 1;
    (a, b)
}

#[test]
fn test_mutable_args() {
    assert_eq!((2, 2), mutable_args(1, 1));
    assert_eq!((2, 2), mutable_args(1, 1));
}

#[allow(unused_mut)]
#[kash]
fn mutable_args_str(mut a: String) -> String {
    a.push_str("-ok");
    a
}

#[test]
fn test_mutable_args_str() {
    assert_eq!("a-ok", mutable_args_str(String::from("a")));
    assert_eq!("a-ok", mutable_args_str(String::from("a")));
}

#[kash::kash(result, ttl = "1")]
fn always_failing() -> Result<String, ()> {
    Err(())
}
