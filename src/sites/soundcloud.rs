use scraper::{Html, Selector};
use tokio::sync::OnceCell;

use crate::error::SiteError;
use crate::id::hit_id;
use crate::model::{Lyrics, SearchHit, Site};

const SITE: Site = Site::SoundCloud;
const CLIENT_ID_MARKER: &str = "client_id:\"";
const MIN_DESCRIPTION_LEN: usize = 100;

async fn bundle_urls(client: &reqwest::Client) -> Result<Vec<String>, SiteError> {
    let response = client
        .get("https://soundcloud.com/")
        .send()
        .await
        .map_err(|source| SiteError::Http {
            site: SITE.label(),
            source,
        })?
        .error_for_status()
        .map_err(|source| SiteError::Http {
            site: SITE.label(),
            source,
        })?;

    let body = response.text().await.map_err(|source| SiteError::Http {
        site: SITE.label(),
        source,
    })?;

    let selector = Selector::parse("script[src]").map_err(|e| SiteError::Selector {
        site: SITE.label(),
        message: format!("{e:?}"),
    })?;

    let document = Html::parse_document(&body);
    let urls = document
        .select(&selector)
        .filter_map(|el| el.value().attr("src"))
        .filter(|src| src.contains("sndcdn.com/assets/") && src.ends_with(".js"))
        .map(str::to_string)
        .collect();
    Ok(urls)
}

fn extract_client_id(js: &str) -> Result<&str, SiteError> {
    let missing = || SiteError::ClientId {
        site: SITE.label(),
    };
    let start = js.find(CLIENT_ID_MARKER).ok_or_else(missing)?;
    let rest = &js[start + CLIENT_ID_MARKER.len()..];
    let end = rest.find('"').ok_or_else(missing)?;
    Ok(&rest[..end])
}

async fn fetch_client_id(client: &reqwest::Client) -> Result<String, SiteError> {
    let bundles = bundle_urls(client).await?;
    for url in bundles {
        let Ok(response) = client.get(&url).send().await else {
            continue;
        };
        let Ok(response) = response.error_for_status() else {
            continue;
        };
        let Ok(body) = response.text().await else {
            continue;
        };
        let Ok(id) = extract_client_id(&body) else {
            continue;
        };
        return Ok(id.to_string());
    }
    Err(SiteError::ClientId {
        site: SITE.label(),
    })
}

async fn client_id(client: &reqwest::Client, cache: &OnceCell<String>) -> Result<String, SiteError> {
    let id = cache.get_or_try_init(|| fetch_client_id(client)).await?;
    Ok(id.clone())
}

async fn get_json(request: reqwest::RequestBuilder) -> Result<serde_json::Value, SiteError> {
    let response = request.send().await.map_err(|source| SiteError::Http {
        site: SITE.label(),
        source,
    })?;
    let response = response.error_for_status().map_err(|source| SiteError::Http {
        site: SITE.label(),
        source,
    })?;
    let body = response.text().await.map_err(|source| SiteError::Http {
        site: SITE.label(),
        source,
    })?;
    serde_json::from_str(&body).map_err(|source| SiteError::Api {
        site: SITE.label(),
        source,
    })
}

pub async fn search(
    client: &reqwest::Client,
    cache: &OnceCell<String>,
    artist: &str,
    title: &str,
    max_hits: usize,
) -> Result<Vec<SearchHit>, SiteError> {
    let cid = client_id(client, cache).await?;
    let query = format!("{artist} {title}");
    let limit = max_hits.to_string();
    let request = client
        .get("https://api-v2.soundcloud.com/search/tracks")
        .query(&[
            ("q", query.as_str()),
            ("client_id", cid.as_str()),
            ("limit", limit.as_str()),
        ]);

    let body = get_json(request).await?;
    let layout_err = || SiteError::Layout {
        site: SITE.label(),
        url: "https://api-v2.soundcloud.com/search/tracks".to_string(),
    };
    let items = body
        .get("collection")
        .and_then(|v| v.as_array())
        .ok_or_else(layout_err)?;

    let mut hits = Vec::new();
    for item in items.iter().take(max_hits) {
        let Some(permalink) = item.get("permalink_url").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(track_title) = item.get("title").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(track_artist) = item
            .get("user")
            .and_then(|user| user.get("username"))
            .and_then(|v| v.as_str())
        else {
            continue;
        };
        let Some(description) = item.get("description").and_then(|v| v.as_str()) else {
            continue;
        };
        if description.trim().chars().count() < MIN_DESCRIPTION_LEN {
            continue;
        }

        hits.push(SearchHit {
            id: hit_id(SITE, permalink),
            site: SITE,
            url: permalink.to_string(),
            title: track_title.to_string(),
            artist: track_artist.to_string(),
        });
    }

    Ok(hits)
}

pub async fn fetch(
    client: &reqwest::Client,
    cache: &OnceCell<String>,
    url: &str,
) -> Result<Lyrics, SiteError> {
    let cid = client_id(client, cache).await?;
    let request = client
        .get("https://api-v2.soundcloud.com/resolve")
        .query(&[("url", url), ("client_id", cid.as_str())]);

    let body = get_json(request).await?;
    let layout_err = || SiteError::Layout {
        site: SITE.label(),
        url: url.to_string(),
    };
    let description = body
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(layout_err)?;

    if description.trim().is_empty() {
        return Err(layout_err());
    }

    Ok(Lyrics {
        site: SITE,
        url: url.to_string(),
        text: description.to_string(),
    })
}
