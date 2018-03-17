// Copyright 2018 Stefan Kroboth
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Storage for persistently saving return values of functions in Redis.
use std::error::Error;
use redis::{self, Commands};
use errors::*;

#[allow(unused_imports)]
use PREFIX;
use PersistentCache;

/// `RedisStorage` struct holds a `redis::Connection` variable.
pub struct RedisStorage {
    con: redis::Connection,
}

impl RedisStorage {
    /// Connects to the Redis server listening at `host` and constructs a new `RedisStorage`
    /// struct.
    ///
    /// This will fail in case there is no redis server running.
    ///
    /// # Examples
    ///
    /// ```
    /// use persistentcache::storage::redis::RedisStorage;
    ///
    /// let s = RedisStorage::new("redis://127.0.0.1").unwrap();
    /// ```
    pub fn new(host: &str) -> Result<Self> {
        let client = redis::Client::open(host)?;
        let con = client.get_connection()?;
        Ok(RedisStorage { con })
    }
}

impl PersistentCache for RedisStorage {
    /// Returns the value within the Redis variable `name`.
    fn get(&self, name: &str) -> Result<Vec<u8>> {
        match self.con.get(name) {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        }
    }

    /// Sets the Redis variable `name` to the array `val` of type `&[u8]`.
    fn set(&self, name: &str, val: &[u8]) -> Result<()> {
        // Yes, this is weird.
        let r: Result<()> = self.con.set(name, val).map_err(|e| e.into());
        r?;
        Ok(())
    }

    /// Delete all variables stored in the Redis database which start with `PREFIX_`.
    fn flush(&self) -> Result<()> {
        let iter: redis::Iter<String> = redis::cmd("KEYS")
            .arg(format!("{}_*", PREFIX))
            .iter(&self.con)?;
        let cmd: &mut redis::Cmd = &mut redis::cmd("DEL");
        // Not a very good looking hack, but I dont know how to figure out whether the iterator is
        // empty or not...
        let mut flushed_vars = 0;
        for bla in iter {
            flushed_vars += 1;
            cmd.arg(bla);
        }
        if flushed_vars > 0 {
            let r: Result<()> = cmd.query(&self.con).map_err(|e| e.into());
            // This is weird.
            r?;
        }
        Ok(())
    }
}
