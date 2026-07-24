use std::cmp::Reverse;

use futures::future::join_all;
use tokio::sync::OnceCell;

use crate::config::Config;
use crate::error::SiteError;
use crate::model::{Lyrics, SearchHit, Site};
use crate::sites::spec::SPECS;
use crate::sites::{scrape, soundcloud};

pub async fn search_all(
    client: &reqwest::Client,
    soundcloud_client_id: &OnceCell<String>,
    artist: &str,
    title: &str,
    config: &Config,
) -> Vec<SearchHit> {
    let scrapes = join_all(
        SPECS
            .iter()
            .map(|spec| scrape::search(client, spec, artist, title)),
    );
    let cloud = soundcloud::search(
        client,
        soundcloud_client_id,
        artist,
        title,
        config.soundcloud_max_hits,
    );
    let (scraped, cloud) = tokio::join!(scrapes, cloud);

    let mut hits = Vec::new();
    for (spec, result) in SPECS.iter().zip(scraped) {
        match result {
            Ok(mut found) => hits.append(&mut found),
            Err(e) => tracing::warn!(error = %e, site = spec.site.label(), "search failed"),
        }
    }
    match cloud {
        Ok(mut found) => hits.append(&mut found),
        Err(e) => tracing::warn!(error = %e, site = Site::SoundCloud.label(), "search failed"),
    }

    hits.sort_by_key(|hit| Reverse(hit.site.reliability()));
    hits.truncate(config.max_hits);
    hits
}

pub async fn fetch(
    client: &reqwest::Client,
    soundcloud_client_id: &OnceCell<String>,
    hit: &SearchHit,
) -> Result<Lyrics, SiteError> {
    if hit.site == Site::SoundCloud {
        return soundcloud::fetch(client, soundcloud_client_id, &hit.url).await;
    }

    let spec = SPECS
        .iter()
        .find(|spec| spec.site == hit.site)
        .ok_or(SiteError::Unsupported {
            site: hit.site.label(),
        })?;
    scrape::fetch(client, spec, &hit.url, &hit.title).await
}
