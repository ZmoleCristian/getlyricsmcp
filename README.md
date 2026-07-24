# getlyricsmcp

An MCP server that finds and fetches song lyrics. **No API keys, no accounts, no
search engine.** It guesses each site's song-page URL directly from the artist and
title you give it, keeps whichever guesses actually resolve, and parses the lyrics
off the page with CSS selectors.

13 sources across 12 languages, all probed in parallel — a search takes about a
second.

## Install

**Cargo** (any platform with a Rust toolchain)

```bash
cargo install getlyricsmcp
```

**AUR** (Arch — yay / paru / makepkg)

```bash
yay -S getlyricsmcp        # build from source
yay -S getlyricsmcp-bin    # prebuilt binary
```

**Homebrew** (macOS arm64, Linux x86_64)

```bash
brew tap zmolecristian/getlyricsmcp https://github.com/ZmoleCristian/getlyricsmcp
brew install getlyricsmcp
```

**Scoop** (Windows)

```powershell
scoop bucket add getlyricsmcp https://github.com/ZmoleCristian/getlyricsmcp
scoop install getlyricsmcp
```

**From source**

```bash
git clone https://github.com/ZmoleCristian/getlyricsmcp
cd getlyricsmcp
cargo install --path .
```

Needs Rust 1.85 or newer (edition 2024).

## Register the MCP server

Claude Code:

```bash
claude mcp add getlyricsmcp -- getlyricsmcp          # this project
claude mcp add -s user getlyricsmcp -- getlyricsmcp  # everywhere
```

Any other MCP client — stdio transport, no arguments, no environment:

```json
{
  "mcpServers": {
    "getlyricsmcp": {
      "type": "stdio",
      "command": "getlyricsmcp",
      "args": []
    }
  }
}
```

## Tools

### `search_lyrics(artist, title)`

Probes every source in parallel and returns the candidates that resolved, sorted
most-reliable-first. Each hit carries an `id` you pass to `get_lyrics`.

```
7bd4e0c9f21a3f55 | genius        | Nirvana - Lithium | https://genius.com/Nirvana-lithium-lyrics
b1c8a37e0d4f2210 | versuri.ro    | Nirvana - Lithium | https://www.versuri.ro/versuri/nirvana-lithium/
...
```

Sources that are known to serve a *plausible but wrong* page instead of a 404 are
title-checked during the search, so a hit that made it into the list is expected to
fetch cleanly.

### `get_lyrics(id)`

Fetches and parses the full lyrics text for one hit. Always fetched fresh from the
source — the text is never written to disk or held in a cache.

## How it works

There is no search API behind this. For a given artist and title, each site's URL is
constructed from a template:

```
genius.com      →  /{artist}-{title}-lyrics          hyphenated
azlyrics.com    →  /lyrics/{artist}/{title}.html     no separators at all
tekstowo.pl     →  /{artist}/{title}                 hyphenated
```

The URL is requested; HTTP 200 means the song is probably there, 404 means it isn't.
That's the whole search. It works because lyrics sites use deterministic,
human-readable slugs — and it costs one HEAD-shaped GET per source instead of an API
key and a rate limit.

Three sources answer 200 with the artist's index page rather than 404ing on a bad
guess. For those, the page `<title>` is checked against the requested title and the
candidate is dropped if it doesn't match.

### Sources

| Site | Language | Reliability |
|---|---|---|
| azlyrics.com | English | High |
| genius.com | English (fallback for everything) | High |
| versuri.ro | Romanian | High |
| versuri.us | Romanian | High |
| lyricshare.net | Russian | High |
| tekstowo.pl | Polish | High |
| letras.mus.br | Portuguese | High |
| sarkisozum.gen.tr | Turkish | High |
| klyrics.net | Korean | High |
| paroles.net | French | Medium |
| angolotesti.it | Italian | Medium |
| letras.com | Spanish | Medium |
| soundcloud.com | any | Low |

**High** — a bad guess 404s cleanly, no known false-positive mode.
**Medium** — has a confirmed silent fallback-to-wrong-page mode, caught by the title
check. That check is a heuristic, not a guarantee.
**Low** — not a dedicated lyrics site. SoundCloud is included for artists who publish
lyrics in the track description; candidates whose description is too short to be
lyrics are dropped.

### Adding a source

Every HTML source is one row of data in `SPECS` (`src/sites/spec.rs`) — URL template,
slug flavour per field, CSS selector, extraction mode, and whether the page title
needs verifying:

```rust
SiteSpec {
    site: Site::Tekstowo,
    template: "https://www.tekstowo.pl/{artist}/{title}",
    artist_slug: Slug::Hyphen,
    title_slug: Slug::Hyphen,
    selector: "#songText > .inner-text",
    extract: Extract::First,
    verify_title: false,
},
```

`src/sites/scrape.rs` is the single generic `search`/`fetch` that drives all of them.
Adding a source means adding a row and a `Site` enum variant — no new file, no new
match arm. SoundCloud is the one exception; it's a JSON API rather than scraping and
lives in `src/sites/soundcloud.rs`.

## Configuration

None. Everything lives in `src/config.rs` — timeouts, hit caps, cache TTL, user
agent. Set `RUST_LOG` (e.g. `RUST_LOG=debug`) for tracing output on stderr.

## Scope and caveats

This is a personal lookup tool. It does what you would do yourself with a browser:
open a song page, read it, close it. Concretely:

- **Nothing is stored.** Only hit metadata (title, artist, url, id) is kept, in
  memory, with a short TTL. The lyrics text itself is re-fetched from the source on
  every `get_lyrics` call and never written anywhere.
- **Nothing is redistributed.** There is no server, no database, no cache of lyrics,
  no third party receiving anything.
- **It is best-effort.** A missing hit means the URL guess didn't match that site's
  slug, not that the song isn't there.

Lyrics are copyrighted, and the sites this reads have their own terms of service and
`robots.txt`. Pointing an automated tool at them at any volume, or republishing what
it returns, is on you — check the terms of any source you intend to use. If you want
to drop a source, delete its row from `SPECS`.

## License

0BSD — see [LICENSE](LICENSE).
