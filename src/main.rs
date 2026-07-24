mod cache;
mod config;
mod error;
mod id;
mod mcp;
mod model;
mod sites;

use rmcp::ServiceExt;

use crate::config::Config;
use crate::error::AppError;
use crate::mcp::LyricsServer;

const USAGE: &str = "getlyricsmcp — MCP server for song lyrics, stdio transport.

Takes no arguments: it speaks the Model Context Protocol on stdin/stdout and is
meant to be spawned by an MCP client, not run by hand.

  -V, --version    print version and exit
  -h, --help       print this help and exit

Register it with Claude Code:
  claude mcp add getlyricsmcp -- getlyricsmcp

Tools: search_lyrics(artist, title), get_lyrics(id).
Logging: set RUST_LOG (e.g. RUST_LOG=debug), output goes to stderr.";

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "-V" || arg == "--version") {
        println!("getlyricsmcp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("{USAGE}");
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::new();
    let server = LyricsServer::new(config)?;
    let service = server
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await
        .map_err(rmcp::RmcpError::from)?;
    service.waiting().await.map_err(rmcp::RmcpError::from)?;
    Ok(())
}
