initSidebarItems({"constant":[["PREFIX","Every stored variable is prefixed by this string. Currently, the flush functions depend on this in order to decide which variable to flush from the storage. Keeping track of the used variable internally is not an option because they are persistent and may come from another process."]],"macro":[["cache","Cache a single function call."],["cache_func","Cache an entire function."]],"mod":[["persistentcache","Implementation of the macros `cache!` and `cache_func!`."],["storage","Implementation of different persistent storages. Currently on disk (`FileStorage` and `FileMemoryStorage`) and in Redis (`RedisStorage`)."]],"trait":[["PersistentCache","Traits which need to be implemented by any storage"]]});