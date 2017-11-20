# persistentcache-rs

persistentcache-rs implements to macros `cache!` and `cache_func!` to cache function calls or entire functions.
The implemented storages are persistent and can be shared between processes.
Storages either store on disk (`FileStorage`) or Redis (`RedisStorage`).

The documentation can be found [here](https://stefan-k.github.io/rust/persistentcache)

## History

This crate is inspired by [owls-cache](https://github.com/havoc-io/owls-cache) and its primary goal is to teach myself Rust.
While working on it, I realised that a similar crate already exists: [cached-rs](https://github.com/jaemk/cached).
I've borrowed a couple of ideas from there.
Have a look at it, it looks much more professional than this crate and almost certainly has better developers.
Unfortunately it lacks the 'persistent' part and the caches cannot be shared between processes/threads, but it should be fairly easy to extend it.

## License

Licensed under either of

  * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
  * MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
