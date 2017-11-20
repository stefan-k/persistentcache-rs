//! Storage for persistently saving return values of functions on disk.
extern crate regex;
extern crate fs2;

use std::error::Error;
use std::fs::{File, create_dir_all, remove_file, read_dir};
use std::io::prelude::*;
use std::path::Path;
use self::regex::Regex;
use self::fs2::FileExt;

#[allow(unused_imports)]
use PREFIX;
use PersistentCache;

/// `FileStorage` struct
// pub struct FileStorage<'a> {
pub struct FileStorage {
    path: String,
}

impl FileStorage {
    // impl<'a> FileStorage<'a> {
    /// Creates the `path` directory and returns a `FileStorage` struct.
    ///
    /// # Examples
    ///
    /// ```
    /// use persistentcache::storage::file::FileStorage;
    ///
    /// let s = FileStorage::new(".example_dir").unwrap();
    /// ```
    // pub fn new(path: &'a str) -> Result<Self, Box<Error>> {
    pub fn new(path: &str) -> Result<Self, Box<Error>> {
        create_dir_all(path)?;
        Ok(FileStorage { path: path.to_owned() })
    }
}

// impl<'a> PersistentCache for FileStorage<'a> {
impl PersistentCache for FileStorage {
    /// Returns the value corresponding to the variable `name`.
    fn get(&self, name: &str) -> Result<Vec<u8>, Box<Error>> {
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
                Ok(s.to_vec())
            }
            Err(e) => {
                file.unlock()?;
                Err(e.into())
            }
        }
    }

    /// Writes the data of type `&[u8]` in array `val` to the file corresponding to the variable `name`.
    fn set(&self, name: &str, val: &[u8]) -> Result<(), Box<Error>> {
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
    fn flush(&self) -> Result<(), Box<Error>> {
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
