// Copyright 2018 Stefan Kroboth
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Implementation of the macros `cache!` and `cache_func!`.
use PREFIX;
use PersistentCache;
use errors::*;

/// Cache an entire function.
#[macro_export]
macro_rules! cache_func {
    // Create `RedisStorage` with default prefix
    (Redis, $host:expr, fn $f:ident($($x:ident : $t:ty),*) -> $r:ty $b:block) => {
        cache_func!(Redis, $host, "DEF", fn $f($($x : $t),*) -> $r $b);
    };
    // Create `FileStorage` with default prefix
    (File, $dir:expr, fn $f:ident($($x:ident : $t:ty),*) -> $r:ty $b:block) => {
        cache_func!(File, $dir, "DEF", fn $f($($x : $t),*) -> $r $b);
    };
    // Create `RedisStorage` with provided prefix
    (Redis, $host:expr, $prefix:expr, fn $f:ident($($x:ident : $t:ty),*) -> $r:ty $b:block) => {
        fn $f($($x: $t),*) -> $r {
            lazy_static!{
                // Unfortunately, the `redis` crate requires Mutex to work.
                // May need to look into this in more detail.
                static ref S: ::std::sync::Mutex<::storage::redis::RedisStorage> = ::std::sync::Mutex::new(::storage::redis::RedisStorage::new($host).unwrap());
            };
            cache_func!($f($($x),*), $b, $prefix);
        }
    };
    // Create `FileStorage` with provided prefix
    (File, $dir:expr, $prefix:expr, fn $f:ident($($x:ident : $t:ty),*) -> $r:ty $b:block) => {
        fn $f($($x: $t),*) -> $r {
            lazy_static!{
                // In order to be consistent with `RedisStorage`, `FileStorage` also uses a Mutex.
                // However, it would not be necessary.
                static ref S: ::std::sync::Mutex<::storage::file::FileStorage> = ::std::sync::Mutex::new(::storage::file::FileStorage::new($dir).unwrap());
            };
            cache_func!($f($($x),*), $b, $prefix);
        }
    };
    // internal
    ($f:ident($($x:ident),*), $b:block, $prefix:expr) => {
        use bincode;
        use ::std::hash::{Hash, Hasher};

        let mut s = ::std::collections::hash_map::DefaultHasher::new();
        for item in &[&($($x),*)] {
            item.hash(&mut s);
        }
        let var_name = format!("{}_{}_{}_{:?}", PREFIX, $prefix, stringify!($f), s.finish());
        let result: Vec<u8> = S.lock().unwrap().get(&var_name).unwrap();

        match result.len() {
            0 => {
                let res = {$b};
                S.lock().unwrap().set(&var_name, &bincode::serialize(&res, bincode::Infinite).unwrap()).unwrap();
                return res;
            },
            _ => return bincode::deserialize(&result).unwrap(),
        }
    }
}

/// Cache a single function call.
#[macro_export]
macro_rules! cache {
    // no prefix provided
    ($storage:ident, $func:ident($($x:expr),*)) => {
        cache!($storage, $func($($x),*), "DEF")
    };
    // prefix provided
    ($storage:ident, $func:ident($($x:expr),*), $prefix:expr) => {
        (||{
            use bincode;
            use ::std::hash::{Hash, Hasher};

            let mut s = ::std::collections::hash_map::DefaultHasher::new();
            for item in &[&($($x),*)] {
                item.hash(&mut s);
            }
            let var_name = format!("{}_{}_{}_{:?}", PREFIX, $prefix, stringify!($func), s.finish());

            let result: Vec<u8> = $storage.get(&var_name)?;
            match result.len() {
                0 => {
                    match $func($($x),*) {
                        Ok(res) => {
                            $storage.set(&var_name, &bincode::serialize(&res, bincode::Infinite)?)?;
                            Ok(res)
                        }
                        Err(e) => Err(e)
                    }
                },
                _ => match bincode::deserialize(&result) {
                    Ok(res) => Ok(res),
                    Err(e) => Err(e.into()), // I have no idea what I am doing.
                }
            }
       })()
    }
}
