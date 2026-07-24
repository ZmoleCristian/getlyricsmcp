use std::time::Duration;

use moka::sync::Cache;

use crate::error::CacheError;
use crate::model::SearchHit;

#[derive(Clone)]
pub struct HitCache {
    inner: Cache<String, SearchHit>,
}

impl HitCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Cache::builder().time_to_live(ttl).build(),
        }
    }

    pub fn insert(&self, hit: SearchHit) {
        self.inner.insert(hit.id.clone(), hit);
    }

    pub fn get(&self, id: &str) -> Result<SearchHit, CacheError> {
        self.inner
            .get(id)
            .ok_or(CacheError::NotFound(id.to_string()))
    }
}
