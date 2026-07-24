use scraper::node::Node;
use scraper::{ElementRef, Html, Selector};

use crate::error::SiteError;
use crate::id::hit_id;
use crate::model::{Lyrics, SearchHit};
use crate::sites::slug;
use crate::sites::spec::{Extract, SiteSpec};

const TITLE_SELECTOR: &str = "title";

fn parse_selector(spec: &SiteSpec, raw: &str) -> Result<Selector, SiteError> {
    Selector::parse(raw).map_err(|e| SiteError::Selector {
        site: spec.site.label(),
        message: format!("{e:?}"),
    })
}

fn http_err(spec: &SiteSpec) -> impl Fn(reqwest::Error) -> SiteError {
    let site = spec.site.label();
    move |source| SiteError::Http { site, source }
}

fn layout_err(spec: &SiteSpec, url: &str) -> impl Fn() -> SiteError {
    let site = spec.site.label();
    let url = url.to_string();
    move || SiteError::Layout {
        site,
        url: url.clone(),
    }
}

fn page_title(document: &Html, spec: &SiteSpec, url: &str) -> Result<String, SiteError> {
    let selector = parse_selector(spec, TITLE_SELECTOR)?;
    let element = document
        .select(&selector)
        .next()
        .ok_or_else(layout_err(spec, url))?;
    Ok(element.text().collect())
}

fn lines_of(element: ElementRef) -> Vec<String> {
    element
        .text()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn walk_skipping_noise(element: ElementRef) -> Vec<String> {
    let mut out = Vec::new();
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
            }
            Node::Element(el) => {
                if el.name() == "script" || el.id() == Some("mobile-banner") {
                    continue;
                }
                let Some(child_ref) = ElementRef::wrap(child) else {
                    continue;
                };
                out.extend(walk_skipping_noise(child_ref));
            }
            other => tracing::trace!(node = ?other, "skipped non-content node"),
        }
    }
    out
}

fn bare_div_children(container: ElementRef) -> Vec<String> {
    container
        .children()
        .filter_map(ElementRef::wrap)
        .filter(|el| el.value().name() == "div" && el.value().attrs().count() == 0)
        .flat_map(lines_of)
        .collect()
}

fn sibling_div(anchor: ElementRef) -> Vec<ElementRef> {
    anchor
        .next_siblings()
        .filter_map(ElementRef::wrap)
        .filter(|el| el.value().name() == "div" && el.value().attrs().count() == 0)
        .take(1)
        .collect()
}

fn extract(document: &Html, spec: &SiteSpec, url: &str) -> Result<String, SiteError> {
    let selector = parse_selector(spec, spec.selector)?;
    let missing = layout_err(spec, url);

    let lines: Vec<String> = match spec.extract {
        Extract::First => {
            let element = document.select(&selector).next().ok_or_else(&missing)?;
            lines_of(element)
        }
        Extract::All => document.select(&selector).flat_map(lines_of).collect(),
        Extract::SiblingDiv => {
            let anchor = document.select(&selector).next().ok_or_else(&missing)?;
            let element = sibling_div(anchor).into_iter().next().ok_or_else(&missing)?;
            lines_of(element)
        }
        Extract::BareDivs => {
            let container = document.select(&selector).next().ok_or_else(&missing)?;
            bare_div_children(container)
        }
        Extract::WalkSkipNoise => {
            let container = document.select(&selector).next().ok_or_else(&missing)?;
            walk_skipping_noise(container)
        }
    };

    if lines.is_empty() {
        return Err(missing());
    }
    Ok(lines.join("\n"))
}

async fn get_body(spec: &SiteSpec, response: reqwest::Response) -> Result<String, SiteError> {
    response.text().await.map_err(http_err(spec))
}

pub async fn search(
    client: &reqwest::Client,
    spec: &SiteSpec,
    artist: &str,
    title: &str,
) -> Result<Vec<SearchHit>, SiteError> {
    let url = spec.url(artist, title);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(http_err(spec))?;

    if !response.status().is_success() {
        return Ok(Vec::new());
    }

    if spec.verify_title {
        let body = get_body(spec, response).await?;
        let document = Html::parse_document(&body);
        let found = page_title(&document, spec, &url)?;
        if !slug::title_matches(&found, title) {
            tracing::debug!(site = spec.site.label(), %url, "dropped title mismatch");
            return Ok(Vec::new());
        }
    }

    Ok(vec![SearchHit {
        id: hit_id(spec.site, &url),
        site: spec.site,
        url,
        title: title.to_string(),
        artist: artist.to_string(),
    }])
}

pub async fn fetch(
    client: &reqwest::Client,
    spec: &SiteSpec,
    url: &str,
    expected_title: &str,
) -> Result<Lyrics, SiteError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(http_err(spec))?
        .error_for_status()
        .map_err(http_err(spec))?;

    let body = get_body(spec, response).await?;
    let document = Html::parse_document(&body);

    if spec.verify_title {
        let found = page_title(&document, spec, url)?;
        if !slug::title_matches(&found, expected_title) {
            return Err(SiteError::Mismatch {
                site: spec.site.label(),
                url: url.to_string(),
                expected: expected_title.to_string(),
            });
        }
    }

    Ok(Lyrics {
        site: spec.site,
        url: url.to_string(),
        text: extract(&document, spec, url)?,
    })
}
