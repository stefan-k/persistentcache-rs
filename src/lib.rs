//! Macros for persistently caching function calls
//!
//! The values are cached either in files or on Redis. Two storages, `FileStorage` and `RedisStorage`
//! are provided.
//! Caching is performed based on the function name and function parameters, meaning that for every
//! combination of function and parameters, the returned value is stored in a storage. Subsequent
//! calls of this function with the same parameters are not computed, but instead fetched from the
//! storage. This can lead to an decrease in computing time in case the function call is
//! computationally more expensive than fetching the value from the storage. The storages are
//! persistent (stored on disk) and can be shared between different threads and processes.
//! All Parameters to the function need to be Hashable. The return value needs to be serializeable
//! by the crate `bincode`.
//!
//! # Setup
//!
//! Add the following dependencies to your projet:
//!
//! ```text
//! [dependencies]
//! lazy_static = "*"
//! bincode = "*"
//! persistentcache = "*"
//! ```
//!
//! # Caching function calls with `cache!`
//!
//! The macro `cache!` caches a function call. The advantage of this approach over the macro
//! `cache_func` is that different storages can be used for different calls. Furthermore the
//! function can still be called without caching if desired.
//! However, in case of recursive functions, this will most likely not work as expected because the
//! recursive calls will not be cached.
//! The macro expects the function to return a value of type `Result<T, Box<std::error::Error>>`.
//!
//! ## Example
//!
//! ```
//! #![allow(redundant_closure_call)]
//! extern crate bincode;
//! #[macro_use] extern crate persistentcache;
//! use persistentcache::*;
//!
//! fn add_two(a: u64) -> Result<u64, Box<std::error::Error>> {
//!     println!("Calculating {} + 2...", a);
//!     Ok(a + 2)
//! }
//!
//! fn main() {
//!     let s = storage::redis::RedisStorage::new("redis://127.0.0.1").unwrap();
//!     // Function is called and will print "Calculating 2 + 2..." and "4"
//!     println!("{}", cache!(s, add_two(2)).unwrap());
//!     // Value will be cached from Redis, will only print "4"
//!     println!("{}", cache!(s, add_two(2)).unwrap());
//!     // Function is called and will print "Calculating 3 + 2..." and "5"
//!     println!("{}", cache!(s, add_two(3)).unwrap());
//!     // Value will be cached from Redis, will only print "5"
//!     println!("{}", cache!(s, add_two(3)).unwrap());
//! }
//! ```
//!
//! This will print:
//!
//! ```text
//! Calculating 2 + 2...
//! 4
//! 4
//! Calculating 3 + 2...
//! 5
//! 5
//! ```
//!
//! # Caching a function with `cache_func!`
//!
//! The macro `cache_func!` is wrapped around a function definition and modifies the function such
//! that the function body is executed and the resulting value is both returned and stored in a
//! provided storage in case the given combination of parameters hasn't been evaluated before.
//! Subsequent calls to the function with already evaluated parameters are then fetched from the
//! storage.
//! The advantage of this approach over `cache!` is that the function is modified and hence every
//! call to the function will automatically take care of the caching. Furthermore it works with
//! recursive calls. However, caching cannot be 'turned off' anymore.
//! No assumption about the return type are made in this case. The function returns the same type
//! as the initial function definition.
//!
//! ## Example
//!
//! ```
//! #[macro_use] extern crate lazy_static;
//! #[macro_use] extern crate persistentcache;
//! extern crate bincode;
//! use persistentcache::*;
//!
//! // Either store it in a `FileStorage`...
//! cache_func!(File, "test_dir",
//! fn add_two_file(a: u64) -> u64 {
//!     println!("Calculating {} + 2...", a);
//!     a + 2
//! });
//!
//! // ... or in a `RedisStorage`
//! cache_func!(Redis, "redis://127.0.0.1",
//! fn add_two_redis(a: u64) -> u64 {
//!     println!("Calculating {} + 2...", a);
//!     a + 2
//! });
//!
//! fn main() {
//!     /*// Function is called and will print "Calculating 2 + 2..." and "4"
//!     println!("{}", s, add_two_file(2));
//!     // Value will be cached from Redis, will only print "4"
//!     println!("{}", s, add_two_file(2));
//!     // Function is called and will print "Calculating 3 + 2..." and "5"
//!     println!("{}", s, add_two_redis(3));
//!     // Value will be cached from Redis, will only print "5"
//!     println!("{}", s, add_two_redis(3));*/
//! }
//! ```
//!
//! This will print:
//!
//! ```text
//! Calculating 2 + 2...
//! 4
//! 4
//! Calculating 3 + 2...
//! 5
//! 5
//! ```
//!
//! # Implementing other storages
//!
//! Storages need to implement the `PersistentCache` trait.
//!
//! # Running the tests
//!
//! The tests should be run in a single thread because the Storages are regularly flushed.
//!
//! ```bash
//! cargo test --features clippy -- --test-threads=1
//! ```
//!
//! A Redis server needs to be running and listening at `127.0.0.1` for the tests to work.
//!
//! # History
//!
//! This crate is inspired by [owls-cache](https://github.com/havoc-io/owls-cache) and its primary
//! goal is to teach myself Rust. While working on it, I realised that a similar crate already
//! exists: [cached-rs](https://github.com/jaemk/cached). I've borrowed a couple of ideas from
//! there. Have a look at it, it looks much more professional than this crate and almost certainly
//! has better developers. Unfortunately it lacks the 'persistent' part and the caches cannot be
//! shared between processes/threads, but it should be fairly easy to extend it.
//!
#![recursion_limit = "1024"]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", allow(redundant_closure_call))]
// #![feature(trace_macros)]
// #![feature(log_syntax)]
#![allow(unused_imports)]
#![warn(missing_docs)]
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate bincode;
extern crate redis;
extern crate regex;
extern crate fs2;

mod errors {
    error_chain!{
        foreign_links {
            Redis(::redis::RedisError);
            Regex(::regex::Error);
            IO(::std::io::Error);
            Bincode(::bincode::Error);
        }
    }
}

use errors::*;

#[macro_use]
pub mod persistentcache;
pub mod storage;

/// Every stored variable is prefixed by this string. Currently, the flush functions depend on this
/// in order to decide which variable to flush from the storage. Keeping track of the used variable
/// internally is not an option because they are persistent and may come from another process.
pub const PREFIX: &str = "pc";

/// Traits which need to be implemented by any storage
pub trait PersistentCache {
    /// Return serialized value of variable
    fn get(&self, &str) -> Result<Vec<u8>>;
    /// Set serialized value of variable
    fn set(&self, &str, &[u8]) -> Result<()>;
    /// Flush storage
    fn flush(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    extern crate num;
    use super::*;
    use std::error::Error;
    use self::num::{Num, NumCast};
    use storage::redis::RedisStorage;
    use storage::file::FileStorage;

    fn test_func_1<T: Num + NumCast>(a: T, counter: &mut i64) -> Result<T> {
        *counter += 1;
        let ten: T = NumCast::from(10_i64).unwrap();
        Ok(a * ten)
    }

    fn test_func_2<T: Num>(a: T, b: T, counter: &mut i64) -> Result<T> {
        *counter += 1;
        Ok(a * b)
    }

    fn test_func_3<T: Copy>(a: &[T], counter: &mut i64) -> Result<Vec<T>> {
        *counter += 1;
        Ok(vec![a[1], a[0]])
    }

    fn throw_error() -> Result<()> {
        Err(
            ::std::io::Error::new(::std::io::ErrorKind::Other, "fu").into(),
        )
    }

    #[test]
    fn test_fib() {
        let s = RedisStorage::new("redis://127.0.0.1").unwrap();
        s.flush().unwrap();
        cache_func!(Redis, "redis://127.0.0.1", 
            fn fib(n: u64) -> u64 {
                if n == 0 || n ==1 {
                    return n
                }
                fib(n-1) + fib(n-2)
            });
        assert_eq!(fib(10), 55);
        s.flush().unwrap();
    }

    #[test]
    fn test_func() {
        let s = FileStorage::new("file_test").unwrap();
        s.flush().unwrap();
        cache_func!(File, "test", 
            fn add_two(n: u64) -> u64 {
                n + 2
            });
        assert_eq!(12, add_two(10));
        s.flush().unwrap();
    }

    #[test]
    fn test_redis_storage() {
        let a: i64 = 6;
        let mut counter: i64 = 0;
        let s = RedisStorage::new("redis://127.0.0.1").unwrap();
        s.flush().unwrap();
        assert_eq!(a * 10, test_func_1(a, &mut counter).unwrap());
        assert_eq!(counter, 1);
        assert_eq!(a * 10, cache!(s, test_func_1(a, &mut counter)).unwrap());
        assert_eq!(counter, 2);
        let mut counter: i64 = 1;
        assert_eq!(a * 10, cache!(s, test_func_1(a, &mut counter)).unwrap());
        assert_eq!(counter, 1);
        s.flush().unwrap();
    }

    #[test]
    fn test_file_storage() {
        let a: i64 = 6;
        let mut counter: i64 = 0;
        let s = FileStorage::new("file_test").unwrap();
        s.flush().unwrap();
        assert_eq!(a * 10, test_func_1(a, &mut counter).unwrap());
        assert_eq!(counter, 1);
        assert_eq!(a * 10, cache!(s, test_func_1(a, &mut counter)).unwrap());
        assert_eq!(counter, 2);
        let mut counter: i64 = 1;
        assert_eq!(a * 10, cache!(s, test_func_1(a, &mut counter)).unwrap());
        assert_eq!(counter, 1);
        s.flush().unwrap();
    }

    #[test]
    fn test_hashing() {
        // swapping the indices should change the hashes!
        let a: i64 = 6;
        let b: i64 = 2;
        let mut counter: i64 = 0;
        let s = FileStorage::new("file_test").unwrap();
        s.flush().unwrap();
        assert_eq!(a * b, cache!(s, test_func_2(a, b, &mut counter)).unwrap());
        assert_eq!(counter, 1);
        let mut counter: i64 = 0;
        assert_eq!(a * b, cache!(s, test_func_2(b, a, &mut counter)).unwrap());
        assert_eq!(counter, 1);
    }

    #[test]
    fn test_vectors() {
        let a: Vec<i64> = vec![1, 2, 3];
        let mut counter: i64 = 0;
        let s = FileStorage::new("file_test").unwrap();
        s.flush().unwrap();
        assert_eq!(vec![2, 1], test_func_3(&a, &mut counter).unwrap());
        assert_eq!(counter, 1);
        assert_eq!(
            vec![2, 1],
            cache!(s, test_func_3(&a, &mut counter)).unwrap()
        );
        assert_eq!(counter, 2);
        let mut counter: i64 = 1;
        assert_eq!(
            vec![2, 1],
            cache!(s, test_func_3(&a, &mut counter)).unwrap()
        );
        assert_eq!(counter, 1);
        s.flush().unwrap();
    }

    #[test]
    #[should_panic]
    fn failing_function() {
        let s = FileStorage::new("file_test").unwrap();
        s.flush().unwrap();
        cache!(s, throw_error()).unwrap();
    }
}
