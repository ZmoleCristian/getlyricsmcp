use thiserror::Error;

#[derive(Debug, Error)]
pub enum SiteError {
    #[error("http request to {site} failed: {source}")]
    Http {
        site: &'static str,
        #[source]
        source: reqwest::Error,
    },
    #[error("could not find a lyrics container on the {site} page at {url}")]
    Layout { site: &'static str, url: String },
    #[error("{site} selector is invalid: {message}")]
    Selector { site: &'static str, message: String },
    #[error("{site} response could not be parsed: {source}")]
    Api {
        site: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("could not resolve a {site} client_id")]
    ClientId { site: &'static str },
    #[error("{site} redirected {url} to an unrelated page (expected title {expected:?})")]
    Mismatch {
        site: &'static str,
        url: String,
        expected: String,
    },
    #[error("{site} has no scrape spec registered")]
    Unsupported { site: &'static str },
}

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("no cached search hit for id {0}")]
    NotFound(String),
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("mcp server error: {0}")]
    Mcp(#[from] rmcp::RmcpError),
}
