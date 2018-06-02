// Copyright 2018 Stefan Kroboth
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! # FileMemoryStorage
//!
//! Storage for persistently saving return values of functions on disk.
//! This storage also stores the data in a HashMap in memory. If the data is available in the
//! HashMap, it will be retreived from there, otherwise it will be retreived from disk.
//! Once a value is retreived from disk, it is also stored in the HashMap.

use errors::*;
use fs2::FileExt;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{create_dir_all, read_dir, remove_file, File};
use std::io::prelude::*;
use std::path::Path;

use PersistentCache;
#[allow(unused_imports)]
use PREFIX;

/// `FileMemoryStorage` struct
pub struct FileMemoryStorage {
    /// Indicates where files are saved
    path: String,
    /// HashMap storing all values alongside the disk
    mem: HashMap<String, Vec<u8>>,
}

impl FileMemoryStorage {
    /// Creates the `path` directory and returns a `FileMemoryStorage` struct.
    ///
    /// # Example
    ///
    /// ```
    /// use persistentcache::storage::file_memory::FileMemoryStorage;
    ///
    /// let mut s = FileMemoryStorage::new(".example_dir").unwrap();
    /// ```
    pub fn new(path: &str) -> Result<Self> {
        create_dir_all(path)?;
        Ok(FileMemoryStorage {
            path: path.to_owned(),
            mem: HashMap::new(),
        })
    }
}

impl PersistentCache for FileMemoryStorage {
    /// Returns the value corresponding to the variable `name`.
    /// If it is stored in the hash map, it will retreive it from there, otherwise it will retreive
    /// it from the file system.
    fn get(&mut self, name: &str) -> Result<Vec<u8>> {
        if self.mem.contains_key(&name.to_string()) {
            Ok(self.mem.get(&name.to_string()).unwrap().clone())
        } else {
            let fpath = format!("{}/{}", self.path, name);
            let p = Path::new(&fpath);
            let mut file = match File::open(&p) {
                Err(_) => return Ok(vec![]),
                Ok(f) => f,
            };
            file.lock_exclusive()?;
            let mut s: Vec<u8> = Vec::new();
            match file.read_to_end(&mut s) {
                Ok(_) => {
                    file.unlock()?;
                    // also store in HashMap
                    self.mem.insert(name.to_string(), s.to_vec());
                    Ok(s.to_vec())
                }
                Err(e) => {
                    file.unlock()?;
                    Err(e.into())
                }
            }
        }
    }

    /// Writes the data of type `&[u8]` in array `val` to the file corresponding to the variable `name`.
    fn set(&mut self, name: &str, val: &[u8]) -> Result<()> {
        // Write into hash map
        self.mem.insert(name.to_string(), val.to_vec());

        // Write to file
        let fpath = format!("{}/{}", self.path, name);
        let p = Path::new(&fpath);
        let mut file = match File::create(&p) {
            Err(e) => return Err(e.into()),
            Ok(f) => f,
        };

        file.lock_exclusive()?;
        file.write_all(val)?;
        file.unlock()?;
        Ok(())
    }

    /// Delete all variables stored in `path` (see `new()`) which start with `PREFIX_`.
    fn flush(&mut self) -> Result<()> {
        // clear memory
        self.mem.clear();

        // remove files
        let p = Path::new(&self.path);
        match read_dir(p) {
            Err(e) => return Err(e.into()),
            Ok(iterator) => {
                let re = Regex::new(&format!(r"^{}/{}_", self.path, PREFIX))?;
                for file in iterator {
                    let tmp = file?.path();
                    let f = tmp.to_str().unwrap();
                    if re.is_match(f) {
                        remove_file(f)?
                    }
                }
            }
        }
        Ok(())
    }
}
