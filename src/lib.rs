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
