# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - 2024-xx-yy

### Added

### Changed

### Removed

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

### Removed

- Remove wasm support, since `moka` doesn't support it
- Remove `cache!` declarative macro. Now you can only use attribute macros, like `#[kash]`
- Remove `#[once]` macro. Instead, you can use `#[cache]` with a custom `key` as `bool`, for example
