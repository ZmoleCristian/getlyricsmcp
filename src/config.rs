use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub user_agent: &'static str,
    pub http_timeout: Duration,
    pub cache_ttl: Duration,
    pub max_hits: usize,
    pub soundcloud_max_hits: usize,
}

impl Config {
    pub fn new() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (X11; Linux x86_64; rv:120.0) Gecko/20100101 Firefox/120.0",
            http_timeout: Duration::from_secs(10),
            cache_ttl: Duration::from_secs(300),
            max_hits: 20,
            soundcloud_max_hits: 5,
        }
    }
}
