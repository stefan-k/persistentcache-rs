//! Storage for persistently saving return values of functions in Redis.
extern crate redis;

use std::error::Error;
use self::redis::Commands;

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
    pub fn new(host: &str) -> Result<Self, Box<Error>> {
        let client = redis::Client::open(host)?;
        let con = client.get_connection()?;
        Ok(RedisStorage { con: con })
    }
}

impl PersistentCache for RedisStorage {
    /// Returns the value within the Redis variable `name`.
    fn get(&self, name: &str) -> Result<Vec<u8>, Box<Error>> {
        match self.con.get(name) {
            Ok(res) => Ok(res),
            Err(e) => Err(e.into()),
        }
    }

    /// Sets the Redis variable `name` to the array `val` of type `&[u8]`.
    fn set(&self, name: &str, val: &[u8]) -> Result<(), Box<Error>> {
        // Yes, this is weird.
        let r: Result<(), self::redis::RedisError> = self.con.set(name, val);
        r?;
        Ok(())
    }

    /// Delete all variables stored in the Redis database which start with `PREFIX_`.
    fn flush(&self) -> Result<(), Box<Error>> {
        let iter: redis::Iter<String> = redis::cmd("KEYS").arg(format!("{}_*", PREFIX)).iter(
            &self.con,
        )?;
        let cmd: &mut redis::Cmd = &mut redis::cmd("DEL");
        // Not a very good looking hack, but I dont know how to figure out whether the iterator is
        // empty or not...
        let mut flushed_vars = 0;
        for bla in iter {
            flushed_vars += 1;
            cmd.arg(bla);
        }
        if flushed_vars > 0 {
            let r: Result<(), self::redis::RedisError> = cmd.query(&self.con);
            // This is weird.
            r?;
        }
        Ok(())
    }
}
