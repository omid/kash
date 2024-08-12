#[macro_use]
extern crate kash;

use kash::{
    proc_macro::kash, CanExpire, ExpiringValueCache, Kash, SizedCache, TimedCache, TimedSizedCache,
    UnboundCache,
};
use serial_test::serial;
use std::thread::{self, sleep};
use std::time::Duration;

kash! {
    UNBOUND_FIB;
    fn fib0(n: u32) -> u32 = {
        if n == 0 || n == 1 { return n }
        fib0(n-1) + fib0(n-2)
    }
}

#[test]
fn test_unbound_cache() {
    fib0(20);
    {
        let cache = UNBOUND_FIB.lock().unwrap();
        assert_eq!(21, cache.cache_size());
    }
}

kash! {
    SIZED_FIB: SizedCache<u32, u32> = SizedCache::with_size(3);
    fn fib1(n: u32) -> u32 = {
        if n == 0 || n == 1 { return n }
        fib1(n-1) + fib1(n-2)
    }
}

#[test]
fn test_sized_cache() {
    let last = fib1(20);
    {
        let cache = SIZED_FIB.lock().unwrap();
        assert_eq!(3, cache.cache_size());
        let items = cache.get_order().iter().collect::<Vec<_>>();
        assert_eq!(3, items.len());
        // (arg, result)
        assert_eq!(&(20, last), items[0]);
    }
}

kash! {
    TIMED: TimedCache<u32, u32> = TimedCache::with_lifespan_and_capacity(2, 5);
    fn timed(n: u32) -> u32 = {
        sleep(Duration::new(3, 0));
        n
    }
}

#[test]
fn test_timed_cache() {
    timed(1);
    timed(1);
    {
        let cache = TIMED.lock().unwrap();
        assert_eq!(1, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
    }
    sleep(Duration::new(3, 0));
    timed(1);
    {
        let cache = TIMED.lock().unwrap();
        assert_eq!(2, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
    }
    {
        let mut cache = TIMED.lock().unwrap();
        assert_eq!(2, cache.cache_set_lifespan(1).unwrap());
    }
    timed(1);
    sleep(Duration::new(1, 0));
    timed(1);
    {
        let cache = TIMED.lock().unwrap();
        assert_eq!(3, cache.cache_misses().unwrap());
        assert_eq!(2, cache.cache_hits().unwrap());
    }
}

kash! {
    TIMED_SIZED: TimedSizedCache<u32, u32> = TimedSizedCache::with_size_and_lifespan(3, 2);
    fn timefac(n: u32) -> u32 = {
        sleep(Duration::new(1, 0));
        if n > 1 {
            n * timefac(n - 1)
        } else {
            n
        }
    }
}

#[test]
fn test_timed_sized_cache() {
    timefac(1);
    timefac(1);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(1, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
    }
    sleep(Duration::new(3, 0));
    timefac(1);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(2, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
    }
    {
        let mut cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(2, cache.cache_set_lifespan(1).unwrap());
    }
    timefac(1);
    sleep(Duration::new(1, 0));
    timefac(1);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(3, cache.cache_misses().unwrap());
        assert_eq!(2, cache.cache_hits().unwrap());
    }
    {
        let mut cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(1, cache.cache_set_lifespan(6).unwrap());
    }
    timefac(2);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(4, cache.cache_misses().unwrap());
        assert_eq!(3, cache.cache_hits().unwrap());
    }
    timefac(3);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(5, cache.cache_misses().unwrap());
        assert_eq!(4, cache.cache_hits().unwrap());
    }
    timefac(3);
    timefac(2);
    timefac(1);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(5, cache.cache_misses().unwrap());
        assert_eq!(7, cache.cache_hits().unwrap());
    }
    timefac(4);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(6, cache.cache_misses().unwrap());
        assert_eq!(8, cache.cache_hits().unwrap());
    }
    timefac(6);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(8, cache.cache_misses().unwrap());
        assert_eq!(9, cache.cache_hits().unwrap());
    }
    timefac(1);
    {
        let cache = TIMED_SIZED.lock().unwrap();
        assert_eq!(9, cache.cache_misses().unwrap());
        assert_eq!(9, cache.cache_hits().unwrap());
        assert_eq!(3, cache.cache_size());
    }
}

kash! {
    STRING_CACHE_EXPLICIT: SizedCache<(String, String), String> = SizedCache::with_size(1);
    fn string_1(a: String, b: String) -> String = {
        a + b.as_ref()
    }
}

#[test]
fn test_string_cache() {
    string_1("a".into(), "b".into());
    {
        let cache = STRING_CACHE_EXPLICIT.lock().unwrap();
        assert_eq!(1, cache.cache_size());
    }
}

kash_key! {
    TIMED_CACHE: TimedCache<u32, u32> = TimedCache::with_lifespan_and_capacity(2, 5);
    Key = { n };
    fn timed_2(n: u32) -> u32 = {
        sleep(Duration::new(3, 0));
        n
    }
}

#[test]
fn test_timed_cache_key() {
    timed_2(1);
    timed_2(1);
    {
        let cache = TIMED_CACHE.lock().unwrap();
        assert_eq!(1, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
    }
    sleep(Duration::new(3, 0));
    timed_2(1);
    {
        let cache = TIMED_CACHE.lock().unwrap();
        assert_eq!(2, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
    }
}

kash_key! {
    SIZED_CACHE: SizedCache<String, usize> = SizedCache::with_size(2);
    Key = { format!("{a}{b}") };
    fn sized_key(a: &str, b: &str) -> usize = {
        let size = a.len() + b.len();
        sleep(Duration::new(size as u64, 0));
        size
    }
}

#[test]
fn test_sized_cache_key() {
    sized_key("a", "1");
    sized_key("a", "1");
    {
        let cache = SIZED_CACHE.lock().unwrap();
        assert_eq!(1, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
        assert_eq!(1, cache.cache_size());
    }
    sized_key("a", "1");
    {
        let cache = SIZED_CACHE.lock().unwrap();
        assert_eq!(1, cache.cache_misses().unwrap());
        assert_eq!(2, cache.cache_hits().unwrap());
        assert_eq!(1, cache.cache_size());
    }
    sized_key("a", "2");
    {
        let cache = SIZED_CACHE.lock().unwrap();
        assert_eq!(2, cache.cache_hits().unwrap());
        assert_eq!(2, cache.cache_size());
        assert_eq!(vec!["a2", "a1"], cache.key_order().collect::<Vec<_>>());
        assert_eq!(vec![&2, &2], cache.value_order().collect::<Vec<_>>());
    }
    sized_key("a", "3");
    {
        let cache = SIZED_CACHE.lock().unwrap();
        assert_eq!(2, cache.cache_size());
        assert_eq!(vec!["a3", "a2"], cache.key_order().collect::<Vec<_>>());
        assert_eq!(vec![&2, &2], cache.value_order().collect::<Vec<_>>());
    }
    sized_key("a", "4");
    sized_key("a", "5");
    {
        let cache = SIZED_CACHE.lock().unwrap();
        assert_eq!(2, cache.cache_size());
        assert_eq!(vec!["a5", "a4"], cache.key_order().collect::<Vec<_>>());
        assert_eq!(vec![&2, &2], cache.value_order().collect::<Vec<_>>());
    }
    sized_key("a", "67");
    sized_key("a", "8");
    {
        let cache = SIZED_CACHE.lock().unwrap();
        assert_eq!(2, cache.cache_size());
        assert_eq!(vec!["a8", "a67"], cache.key_order().collect::<Vec<_>>());
        assert_eq!(vec![&2, &3], cache.value_order().collect::<Vec<_>>());
    }
}

kash_key_result! {
    RESULT_CACHE_KEY: UnboundCache<u32, u32> = UnboundCache::new();
    Key = { n };
    fn test_result_key(n: u32) -> Result<u32, ()> = {
        if n < 5 { Ok(n) } else { Err(()) }
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
    {
        let cache = RESULT_CACHE_KEY.lock().unwrap();
        assert_eq!(2, cache.cache_size());
        assert_eq!(2, cache.cache_hits().unwrap());
        assert_eq!(4, cache.cache_misses().unwrap());
    }
}

kash_result! {
    RESULT_CACHE: UnboundCache<u32, u32> = UnboundCache::new();
    fn test_result_no_default(n: u32) -> Result<u32, ()> = {
        if n < 5 { Ok(n) } else { Err(()) }
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
    {
        let cache = RESULT_CACHE.lock().unwrap();
        assert_eq!(2, cache.cache_size());
        assert_eq!(2, cache.cache_hits().unwrap());
        assert_eq!(4, cache.cache_misses().unwrap());
    }
}

kash_control! {
    CONTROL_CACHE: UnboundCache<String, String> = UnboundCache::new();
    Key = { input.to_owned() };
    PostGet(kash_val) = return Ok(kash_val.clone());
    PostExec(body_result) = {
        match body_result {
            Ok(v) => v,
            Err(e) => return Err(e),
        }
    };
    Set(set_value) = set_value.clone();
    Return(return_value) = {
        println!("{return_value}");
        Ok(return_value)
    };
    fn can_fail(input: &str) -> Result<String, String> = {
        let len = input.len();
        if len < 3 { Ok(format!("{input}-{len}")) }
        else { Err("too big".to_string()) }
    }
}

#[test]
fn test_can_fail() {
    assert_eq!(can_fail("ab"), Ok("ab-2".to_string()));
    assert_eq!(can_fail("abc"), Err("too big".to_string()));
    {
        let cache = CONTROL_CACHE.lock().unwrap();
        assert_eq!(2, cache.cache_misses().unwrap());
    }
    assert_eq!(can_fail("ab"), Ok("ab-2".to_string()));
    {
        let cache = CONTROL_CACHE.lock().unwrap();
        assert_eq!(1, cache.cache_hits().unwrap());
    }
}

kash_key! {
    SIZED_KEY_RESULT_CACHE: SizedCache<String, String> = SizedCache::with_size(2);
    Key = { format!("{a}/{b}") };
    fn slow_small_cache(a: &str, b: &str) -> String = {
        sleep(Duration::new(1, 0));
        format!("{a}:{b}")
    }
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
    // at this point, the cache should have a size of one since the keys are the same
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
    {
        let cache = PROC_KASH_RESULT.lock().unwrap();
        assert_eq!(2, cache.cache_size());
        assert_eq!(2, cache.cache_hits().unwrap());
        assert_eq!(4, cache.cache_misses().unwrap());
    }
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
    {
        let cache = PROC_KASH_OPTION.lock().unwrap();
        assert_eq!(3, cache.cache_size());
        assert_eq!(3, cache.cache_hits().unwrap());
        assert_eq!(5, cache.cache_misses().unwrap());
    }
}

kash_result! {
    RESULT_CACHE_RETARM: UnboundCache<u32, u32> = UnboundCache::new();
    fn test_result_missing_result_arm(n: u32) -> Result<u32, ()> = {
        Ok(n)
    }
}

kash_key_result! {
    RESULT_CACHE_KEY_RETARM: UnboundCache<u32, u32> = UnboundCache::new();
    Key = { n };
    fn test_result_key_missing_result_arm(n: u32) -> Result<u32, ()> = {
        Ok(n)
    }
}

#[kash(size = 1, time = 1)]
fn proc_timed_sized_sleeper(n: u64) -> u64 {
    sleep(Duration::new(1, 0));
    n
}

#[test]
fn test_proc_timed_sized_cache() {
    proc_timed_sized_sleeper(1);
    proc_timed_sized_sleeper(1);
    {
        let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
        assert_eq!(1, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
    }
    // sleep to expire the one entry
    sleep(Duration::new(1, 0));
    proc_timed_sized_sleeper(1);
    {
        let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
        assert_eq!(2, cache.cache_misses().unwrap());
        assert_eq!(1, cache.cache_hits().unwrap());
        assert_eq!(cache.key_order().collect::<Vec<_>>(), vec![&1]);
    }
    // sleep to expire the one entry
    sleep(Duration::new(1, 0));
    {
        let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
        assert!(cache.key_order().next().is_none());
    }
    proc_timed_sized_sleeper(1);
    proc_timed_sized_sleeper(1);
    {
        let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
        assert_eq!(3, cache.cache_misses().unwrap());
        assert_eq!(2, cache.cache_hits().unwrap());
        assert_eq!(cache.key_order().collect::<Vec<_>>(), vec![&1]);
    }
    // lru size is 1, so this new thing evicts the existing key
    proc_timed_sized_sleeper(2);
    {
        let cache = PROC_TIMED_SIZED_SLEEPER.lock().unwrap();
        assert_eq!(4, cache.cache_misses().unwrap());
        assert_eq!(2, cache.cache_hits().unwrap());
        assert_eq!(cache.key_order().collect::<Vec<_>>(), vec![&2]);
    }
}

#[kash(wrap_return)]
fn kash_return_flag(n: i32) -> kash::Return<i32> {
    kash::Return::new(n)
}

#[test]
fn test_kash_return_flag() {
    let r = kash_return_flag(1);
    assert!(!r.was_cached);
    assert_eq!(*r, 1);
    let r = kash_return_flag(1);
    assert!(r.was_cached);
    // derefs to inner
    assert_eq!(*r, 1);
    assert!(r.is_positive());
    {
        let cache = KASH_RETURN_FLAG.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }
}

#[kash(result, wrap_return)]
fn kash_return_flag_result(n: i32) -> Result<kash::Return<i32>, ()> {
    if n == 10 {
        return Err(());
    }
    Ok(kash::Return::new(n))
}

#[test]
fn test_kash_return_flag_result() {
    let r = kash_return_flag_result(1).unwrap();
    assert!(!r.was_cached);
    assert_eq!(*r, 1);
    let r = kash_return_flag_result(1).unwrap();
    assert!(r.was_cached);
    // derefs to inner
    assert_eq!(*r, 1);
    assert!(r.is_positive());

    let r = kash_return_flag_result(10);
    assert!(r.is_err());
    {
        let cache = KASH_RETURN_FLAG_RESULT.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(2));
    }
}

#[kash(option, wrap_return)]
fn kash_return_flag_option(n: i32) -> Option<kash::Return<i32>> {
    if n == 10 {
        return None;
    }
    Some(kash::Return::new(n))
}

#[test]
fn test_kash_return_flag_option() {
    let r = kash_return_flag_option(1).unwrap();
    assert!(!r.was_cached);
    assert_eq!(*r, 1);
    let r = kash_return_flag_option(1).unwrap();
    assert!(r.was_cached);
    // derefs to inner
    assert_eq!(*r, 1);
    assert!(r.is_positive());

    let r = kash_return_flag_option(10);
    assert!(r.is_none());
    {
        let cache = KASH_RETURN_FLAG_OPTION.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(2));
    }
}

#[kash(size = 2)]
fn kash_smartstring(s: smartstring::alias::String) -> smartstring::alias::String {
    if s == "very stringy" {
        smartstring::alias::String::from("equal")
    } else {
        smartstring::alias::String::from("not equal")
    }
}

#[test]
fn test_kash_smartstring() {
    let mut string = smartstring::alias::String::new();
    string.push_str("very stringy");
    assert_eq!("equal", kash_smartstring(string.clone()));
    {
        let cache = KASH_SMARTSTRING.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    assert_eq!("equal", kash_smartstring(string.clone()));
    {
        let cache = KASH_SMARTSTRING.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    let string = smartstring::alias::String::from("also stringy");
    assert_eq!("not equal", kash_smartstring(string));
    {
        let cache = KASH_SMARTSTRING.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(2));
    }
}

#[kash(
    size = 2,
    key = "smartstring::alias::String",
    convert = r#"{ smartstring::alias::String::from(s) }"#
)]
fn kash_smartstring_from_str(s: &str) -> bool {
    s == "true"
}

#[test]
fn test_kash_smartstring_from_str() {
    assert!(kash_smartstring_from_str("true"));
    {
        let cache = KASH_SMARTSTRING_FROM_STR.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    assert!(kash_smartstring_from_str("true"));
    {
        let cache = KASH_SMARTSTRING_FROM_STR.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    assert!(!kash_smartstring_from_str("false"));
    {
        let cache = KASH_SMARTSTRING_FROM_STR.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(2));
    }
}

#[kash(
    time = 1,
    time_refresh,
    key = "String",
    convert = r#"{ String::from(s) }"#
)]
fn kash_timed_refresh(s: &str) -> bool {
    s == "true"
}

#[test]
fn test_kash_timed_refresh() {
    assert!(kash_timed_refresh("true"));
    {
        let cache = KASH_TIMED_REFRESH.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    assert!(kash_timed_refresh("true"));
    {
        let cache = KASH_TIMED_REFRESH.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_refresh("true"));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_refresh("true"));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_refresh("true"));
    {
        let cache = KASH_TIMED_REFRESH.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(4));
        assert_eq!(cache.cache_misses(), Some(1));
    }
}

#[kash(
    size = 2,
    time = 1,
    time_refresh,
    key = "String",
    convert = r#"{ String::from(s) }"#
)]
fn kash_timed_sized_refresh(s: &str) -> bool {
    s == "true"
}

#[test]
fn test_kash_timed_sized_refresh() {
    assert!(kash_timed_sized_refresh("true"));
    {
        let cache = KASH_TIMED_SIZED_REFRESH.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    assert!(kash_timed_sized_refresh("true"));
    {
        let cache = KASH_TIMED_SIZED_REFRESH.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_refresh("true"));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_refresh("true"));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_refresh("true"));
    {
        let cache = KASH_TIMED_SIZED_REFRESH.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(4));
        assert_eq!(cache.cache_misses(), Some(1));
    }
}

#[kash(
    size = 2,
    time = 1,
    time_refresh,
    key = "String",
    convert = r#"{ String::from(s) }"#
)]
fn kash_timed_sized_refresh_prime(s: &str) -> bool {
    s == "true"
}

#[test]
fn test_kash_timed_sized_refresh_prime() {
    assert!(kash_timed_sized_refresh_prime("true"));
    {
        let cache = KASH_TIMED_SIZED_REFRESH_PRIME.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }
    assert!(kash_timed_sized_refresh_prime("true"));
    {
        let cache = KASH_TIMED_SIZED_REFRESH_PRIME.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_refresh_prime_prime_cache("true"));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_refresh_prime_prime_cache("true"));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_refresh_prime_prime_cache("true"));

    // stats unchanged (other than this new hit) since we kept priming
    assert!(kash_timed_sized_refresh_prime("true"));
    {
        let cache = KASH_TIMED_SIZED_REFRESH_PRIME.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(2));
        assert_eq!(cache.cache_misses(), Some(1));
    }
}

#[kash(size = 2, time = 1, key = "String", convert = r#"{ String::from(s) }"#)]
fn kash_timed_sized_prime(s: &str) -> bool {
    s == "true"
}

#[test]
fn test_kash_timed_sized_prime() {
    assert!(kash_timed_sized_prime("true"));
    {
        let cache = KASH_TIMED_SIZED_PRIME.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }
    assert!(kash_timed_sized_prime("true"));
    {
        let cache = KASH_TIMED_SIZED_PRIME.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_prime_prime_cache("true"));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_prime_prime_cache("true"));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(kash_timed_sized_prime_prime_cache("true"));

    // stats unchanged (other than this new hit) since we kept priming
    assert!(kash_timed_sized_prime("true"));
    {
        let mut cache = KASH_TIMED_SIZED_PRIME.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(2));
        assert_eq!(cache.cache_misses(), Some(1));
        assert!(cache.cache_size() > 0);
        std::thread::sleep(std::time::Duration::from_millis(1000));
        cache.flush();
        assert_eq!(cache.cache_size(), 0);
    }
}

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

#[derive(Clone)]
pub struct NewsArticle {
    slug: String,
    is_expired: bool,
}

impl CanExpire for NewsArticle {
    fn is_expired(&self) -> bool {
        self.is_expired
    }
}

const EXPIRED_SLUG: &str = "expired_slug";
const UNEXPIRED_SLUG: &str = "unexpired_slug";

#[kash(
    ty = "ExpiringValueCache<String, NewsArticle>",
    create = "{ ExpiringValueCache::with_size(3) }",
    result
)]
fn fetch_article(slug: String) -> Result<NewsArticle, ()> {
    match slug.as_str() {
        EXPIRED_SLUG => Ok(NewsArticle {
            slug: String::from(EXPIRED_SLUG),
            is_expired: true,
        }),
        UNEXPIRED_SLUG => Ok(NewsArticle {
            slug: String::from(UNEXPIRED_SLUG),
            is_expired: false,
        }),
        _ => Err(()),
    }
}

#[test]
#[serial(ExpiringCacheTest)]
fn test_expiring_value_expired_article_returned_with_miss() {
    {
        let mut cache = FETCH_ARTICLE.lock().unwrap();
        cache.cache_reset();
        cache.cache_reset_metrics();
    }
    let expired_article = fetch_article(EXPIRED_SLUG.to_string());

    assert!(expired_article.is_ok());
    assert_eq!(EXPIRED_SLUG, expired_article.unwrap().slug.as_str());

    // The article was fetched due to a cache miss and the result kash.
    {
        let cache = FETCH_ARTICLE.lock().unwrap();
        assert_eq!(1, cache.cache_size());
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    let _ = fetch_article(EXPIRED_SLUG.to_string());

    // The article was fetched again as it had expired.
    {
        let cache = FETCH_ARTICLE.lock().unwrap();
        assert_eq!(1, cache.cache_size());
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(2));
    }
}

#[test]
#[serial(ExpiringCacheTest)]
fn test_expiring_value_unexpired_article_returned_with_hit() {
    {
        let mut cache = FETCH_ARTICLE.lock().unwrap();
        cache.cache_reset();
        cache.cache_reset_metrics();
    }
    let unexpired_article = fetch_article(UNEXPIRED_SLUG.to_string());

    assert!(unexpired_article.is_ok());
    assert_eq!(UNEXPIRED_SLUG, unexpired_article.unwrap().slug.as_str());

    // The article was fetched due to a cache miss and the result kash.
    {
        let cache = FETCH_ARTICLE.lock().unwrap();
        assert_eq!(1, cache.cache_size());
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    let kash_article = fetch_article(UNEXPIRED_SLUG.to_string());
    assert!(kash_article.is_ok());
    assert_eq!(UNEXPIRED_SLUG, kash_article.unwrap().slug.as_str());

    // The article was not fetched but returned as a hit from the cache.
    {
        let cache = FETCH_ARTICLE.lock().unwrap();
        assert_eq!(1, cache.cache_size());
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }
}

#[kash::proc_macro::kash(result, time = 1, result_fallback)]
fn always_failing() -> Result<String, ()> {
    Err(())
}

#[test]
fn test_result_fallback() {
    assert!(always_failing().is_err());
    {
        let cache = ALWAYS_FAILING.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(0));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    // Pretend it succeeded once
    ALWAYS_FAILING
        .lock()
        .unwrap()
        .cache_set((), "abc".to_string());
    assert_eq!(always_failing(), Ok("abc".to_string()));
    {
        let cache = ALWAYS_FAILING.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(1));
    }

    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Even though the cache should've expired, the `result_fallback` flag means it refreshes the cache with the last valid result
    assert_eq!(always_failing(), Ok("abc".to_string()));
    {
        let cache = ALWAYS_FAILING.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(1));
        assert_eq!(cache.cache_misses(), Some(2));
    }

    assert_eq!(always_failing(), Ok("abc".to_string()));
    {
        let cache = ALWAYS_FAILING.lock().unwrap();
        assert_eq!(cache.cache_hits(), Some(2));
        assert_eq!(cache.cache_misses(), Some(2));
    }
}
