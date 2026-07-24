use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::OnceCell;

use crate::cache::HitCache;
use crate::config::Config;
use crate::sites;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchLyricsArgs {
    pub artist: String,
    pub title: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLyricsArgs {
    pub id: String,
}

#[derive(Clone)]
pub struct LyricsServer {
    tool_router: ToolRouter<Self>,
    http: reqwest::Client,
    config: Arc<Config>,
    cache: HitCache,
    soundcloud_client_id: Arc<OnceCell<String>>,
}

impl LyricsServer {
    pub fn new(config: Config) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder()
            .user_agent(config.user_agent)
            .timeout(config.http_timeout)
            .build()?;
        let cache = HitCache::new(config.cache_ttl);
        Ok(Self {
            tool_router: Self::tool_router(),
            http,
            config: Arc::new(config),
            cache,
            soundcloud_client_id: Arc::new(OnceCell::new()),
        })
    }
}

#[tool_router(router = tool_router)]
impl LyricsServer {
    #[tool(
        name = "search_lyrics",
        description = "Best-effort search for a song across many lyrics sites by guessing each site's URL from artist+title — no API keys, no real search engine. All sites are probed in parallel; candidates that resolve to an unrelated page are dropped, so every returned hit is expected to fetch cleanly. Sorted most-reliable-first, each with an id to pass to get_lyrics."
    )]
    async fn search_lyrics(
        &self,
        p: Parameters<SearchLyricsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let hits = sites::search_all(
            &self.http,
            &self.soundcloud_client_id,
            &p.0.artist,
            &p.0.title,
            &self.config,
        )
        .await;

        for hit in &hits {
            self.cache.insert(hit.clone());
        }

        let summary = if hits.is_empty() {
            "no hits".to_string()
        } else {
            hits.iter()
                .map(|h| {
                    format!(
                        "{} | {} | {} - {} | {}",
                        h.id,
                        h.site.label(),
                        h.artist,
                        h.title,
                        h.url
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let structured = json!({ "hits": hits });

        Ok(CallToolResult {
            content: vec![Content::text(summary)],
            structured_content: Some(structured),
            is_error: None,
            meta: None,
        })
    }

    #[tool(
        name = "get_lyrics",
        description = "Fetch and parse the full lyrics text for a hit id returned by search_lyrics. Always fetched fresh, never cached."
    )]
    async fn get_lyrics(&self, p: Parameters<GetLyricsArgs>) -> Result<CallToolResult, ErrorData> {
        let hit = self.cache.get(&p.0.id).map_err(|e| {
            tracing::warn!(error = %e, "get_lyrics cache miss");
            ErrorData::invalid_params(e.to_string(), None)
        })?;

        let lyrics = sites::fetch(&self.http, &self.soundcloud_client_id, &hit)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "get_lyrics fetch failed");
                ErrorData::internal_error(e.to_string(), None)
            })?;

        let structured = json!(lyrics);

        Ok(CallToolResult {
            content: vec![Content::text(lyrics.text)],
            structured_content: Some(structured),
            is_error: None,
            meta: None,
        })
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for LyricsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "getlyricsmcp".into(),
                title: Some("getlyricsmcp — search and fetch song lyrics".into()),
                version: env!("CARGO_PKG_VERSION").into(),
                website_url: None,
                icons: None,
            },
            instructions: Some(
                "Personal, local-use lyrics lookup, no API keys. search_lyrics(artist, title) \
                 guesses each site's URL from the given artist/title, probes every source in \
                 parallel and returns whichever candidates actually resolve to the right song, \
                 sorted most-reliable-first; get_lyrics(id) fetches \
                 and parses the full text fresh (never cached). Sources: azlyrics.com and \
                 genius.com (English), versuri.ro and versuri.us (Romanian), lyricshare.net \
                 (Russian), paroles.net (French), tekstowo.pl (Polish), letras.mus.br \
                 (Portuguese), angolotesti.it (Italian), letras.com (Spanish), \
                 sarkisozum.gen.tr (Turkish), klyrics.net (Korean), and soundcloud.com track \
                 descriptions (for artists who publish lyrics there, e.g. ytcracker; hits with \
                 a near-empty description are filtered out)."
                    .into(),
            ),
        }
    }
}
