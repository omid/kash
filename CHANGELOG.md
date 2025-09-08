# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2025-xx-yy

### Added

### Changed

### Removed

## [0.6.0] - 2025-09-08

### Changed

- Update dependencies
- `disk::new`, `RedisCacheBuilder::new`, `RedisCacheBuilder::set_namespace` and `RedisCacheBuilder::set_prefix` only accepts `&str` now.

## [0.5.0] - 2024-10-28

### Changed

- Prefixed the variable names internally. Previously, naming your function variables as `key` or `result` could cause conflicts, but now the chances of collisions are reduced.
- When your functions return `Result`, errors must also be `Clone`able.
- Improve `moka` performance.
- Upgrade to edition 2024, so MSRV is 1.85

### Removed

- All memory calls are synced, so `sync_writes` flag is removed.
- Removed support of `async_std`. And `redis_async_std` feature flag.

## [0.4.0] - 2024-10-21

### Changed

- Now we use msgpack to store data in Redis, and not JSON.

  It's considered as a breaking change, since it will return error for existing caches. Because of this, the default Redis namespace has also changed.
  And you may need to clean the Redis cache manually.

### Removed

- Many `clone`s are remove from the Redis integration. So it's expected to becomes a little faster.
- Remove the dependency to `serde_json`

## [0.3.0] - 2024-10-18

### Added

- Add support for LFU caching, alongside the existing LRU cache

### Changed

- This version encounters a breaking change, because from this version on, the default algorithm is LFU.

  If you want to use LRU, you need to pass `eviction_policy="lru"`.

## [0.2.0] - 2024-10-17

### Changed

- Type of `ttl` in `kash` is string. So you can pass functions or consts.
- All `disk` and `redis` specific attributes, went inside parentheses. Like `#[kash(disk(dir = "/dir/"))]` instead of `#[kash(disk, disk_dir = "/dir/")]`
- Change `disk_dir` attribute to `dir`
- Change `cache_prefix_block` attribute to `prefix_block`
- Now `key` has two children elements, `ty` and `expr`. Basically `ty` is the old value of `key` and `expr` is the value of `convert`.

### Removed

- Remove `io_kash`, instead you can simply use `kash`
- `convert` attribute has been removed from the root of configurations. Use `expr` attribute inside `key` attribute. 

## [0.1.2] - 2024-10-14

### Removed

- Remove `map_error` param. Instead, you can impl `From<DiskCacheError>` or `From<RedisCacheError>` for your result.

## [0.1.1] - 2024-10-14

### Removed

- Remove `wrap_result` flag

## [0.1.0] - 2024-10-14

### Added

- Fork from the source project: https://github.com/jaemk/cached/
- Organize and cleanup some codes
- Set MSRV to 1.76
- Support functions inside `impl`
- Add edition 2021

### Changed

- `time` change to `ttl`
- Change custom implementation of memory cache and use `moka`

### Removed

- Remove wasm support, since `moka` doesn't support it
- Remove `cache!` declarative macro. Now you can only use attribute macros, like `#[kash]`
- Remove `#[once]` macro. Instead, you can use `#[cache]` with a custom `key` as `bool`, for example
- It's not possible anymore to use custom cache definitions, you may need to use `cached` library instead
