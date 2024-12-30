use pgrx::prelude::*;
use std::{collections::HashMap, sync::Mutex};

pgrx::pg_module_magic!();

lazy_static::lazy_static! {
    static ref QUERY_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[pg_extern]
fn execute_with_cache(query: &str) -> String {
    // let mut result;
    let mut unlocked_cache = QUERY_CACHE.lock().unwrap();

    let result = match unlocked_cache.get(query) {
        Some(r) => r.clone(),
        None => {
            let query_result = match Spi::get_one::<String>(query) {
                Ok(Some(r)) => r,
                Ok(None) => "".to_string(),
                Err(e) => {
                    eprintln!(
                        "[ERROR] Failed to execute query: \"{}\"\n[DETAILS] Error: {:?}",
                        query, e
                    );
                    e.to_string()
                }
            };
            unlocked_cache.insert(query.to_string(), query_result.clone());
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
