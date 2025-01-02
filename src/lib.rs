use lru::LruCache;
use pgrx::{prelude::*, Json};
use std::{num::NonZero, sync::Mutex};

pgrx::pg_module_magic!();

lazy_static::lazy_static! {
    static ref CACHE_SIZE: usize = 100;
    static ref QUERY_CACHE: Mutex<LruCache<String, String>> = Mutex::new(LruCache::new(NonZero::new(*CACHE_SIZE).unwrap()));
}

#[pg_extern]
fn execute_with_cache(query: &str) -> String {
    let mut unlocked_cache = QUERY_CACHE.lock().unwrap();

    let result = match unlocked_cache.get(query) {
        Some(r) => r.clone(),
        None => {
            let query_result =
                match Spi::get_one::<Json>(&format!("SELECT json_agg(t) FROM ({}) t", query)) {
                    Ok(Some(json)) => json.0.to_string(),
                    Ok(None) => "".to_string(),
                    Err(e) => {
                        eprintln!(
                            "[ERROR] Failed to execute query: \"{}\"\n[DETAILS] Error: {:?}",
                            query, e
                        );
                        e.to_string()
                    }
                };
            unlocked_cache.put(query.to_string(), query_result.clone());
            query_result
        }
    };

    result
}

#[pg_extern]
fn clear_cache() -> &'static str {
    let mut cache = QUERY_CACHE.lock().expect("Failed to lock cache");
    cache.clear();
    "Query cache cleared."
}

#[pg_extern]
fn set_cache_size(size_str: &str) -> &'static str {
    let size = match size_str.parse::<i32>() {
        Ok(s) => s,
        Err(_) => panic!("Error: Cache size must be a number."), // 数値チェック
    };

    // 0以下の場合は更新しない
    if size <= 0 {
        panic!("Error: Cache size must be greater than 0.");
    }

    let mut cache = QUERY_CACHE.lock().unwrap();
    *cache = LruCache::new(NonZero::new(size as usize).unwrap());
    "Cache size updated."
}

// test
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    // テスト専用のキャッシュ確認関数
    fn is_cached(query: &str) -> bool {
        QUERY_CACHE.lock().unwrap().get(query).is_some()
    }

    #[pg_test]
    fn test_execute_with_cache() {
        let query = "SELECT 1 AS col1, 2 AS col2, 3 AS col3";
        // キャッシュにデータがないことを確認
        assert!(!is_cached(query), "The query should not be cached");
        // クエリ実行
        let result = crate::execute_with_cache(query);
        assert_eq!(result, "[{\"col1\":1,\"col2\":2,\"col3\":3}]");
        // キャッシュにデータが載っていることを確認
        assert!(is_cached(query), "The query should be cached");
    }

    #[pg_test]
    fn test_clear_cache() {
        let query = "SELECT 1";
        crate::execute_with_cache(query);
        assert!(is_cached(query), "The query should be cached");

        crate::clear_cache();
        assert!(!is_cached(query), "The cache should be cleared");
    }

    #[pg_test]
    fn test_cache_eviction() {
        crate::set_cache_size("2");
        let queries = vec!["SELECT 1", "SELECT 2", "SELECT 3"];

        for query in &queries {
            crate::execute_with_cache(query);
        }

        // 2,3番目のクエリがキャッシュされていることを確認
        assert!(is_cached("SELECT 2"), "The second query should be cached");
        assert!(
            is_cached("SELECT 3"),
            "The third query should be cached (cache eviction)"
        );
        // 最初のクエリがキャッシュから削除されていることを確認
        assert!(
            !is_cached("SELECT 1"),
            "The first query should be evicted from the cache"
        );
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
