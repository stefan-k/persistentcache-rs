//! Implementation of different persistent storages. Currently on disk (`FileStorage`) and in Redis
//! (`RedisStorage`).

/// `RedisStorage`
pub mod redis;
/// `FileStorage`
pub mod file;
