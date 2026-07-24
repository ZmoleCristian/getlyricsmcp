use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::model::Site;

pub fn hit_id(site: Site, url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    site.label().hash(&mut hasher);
    url.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
