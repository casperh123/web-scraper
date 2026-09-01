# Crawler Demo

A small Rust workspace that crawls Danish (`.dk`) websites, records basic
performance metrics for each domain, and exposes the results through a REST
API.

The project is split into three crates:

| Crate       | Purpose                                                              |
|-------------|-----------------------------------------------------------------------|
| `crawler`   | Async web crawler that discovers and crawls `.dk` domains             |
| `database`  | Shared `sea-orm` data layer (models, connection, queries)             |
| `api`       | `actix-web` HTTP API that serves crawled website data                 |

## How it works

1. **Seeding** — `crawler::seeds::get_seeds` provides a curated list of
   Danish news, government, retail, and community sites to start from.
2. **Crawling** — `crawl_from_seed` fans out to `crawl_domain`, which walks
   each site's internal links (breadth-first, tracked with a Bloom filter to
   avoid re-visiting paths), parses `<a href>` tags with `tl`, and follows
   links up to a cap of 50,000 pages per domain. Up to 300 domains are
   crawled concurrently via a semaphore.
3. **URL filtering** — `url_rules` decides whether a discovered link is worth
   crawling:
   - `filter.rs` — must resolve to a `.dk` host, must not have a URL
     fragment, must pass the query and extension checks.
   - `query.rs` — rejects URLs whose query string contains parameters other
     than a known set of "navigational" params (`page`, `id`, `category`,
     `lang`, etc.), to avoid crawling infinite parameter combinations.
   - `extension.rs` — rejects links pointing at non-HTML assets (images,
     archives, media, stylesheets, scripts, feeds).
4. **Domain discovery** — links to a different host than the one currently
   being crawled are sent to `discovery::filter_domains`, which
   deduplicates hosts (ignoring a `www.` prefix) and feeds new domains back
   into the crawl queue.
5. **Persistence** — once a domain's crawl finishes, its URL, average
   time-to-first-byte, and number of links crawled are sent to
   `save_websites` in `main.rs`, which writes the result to Postgres via the
   `database` crate.
6. **API** — the `api` crate exposes a paginated `GET /websites` endpoint
   backed by the same Postgres table, ordered by number of links crawled.

## Project layout

```
.
├── crawler/                 # Crawler binary + library
│   └── src/
│       ├── crawler/         # Crawl loop, domain discovery, seeds
│       └── url_rules/       # Filtering logic (query, extension, host)
├── database/                 # Shared sea-orm models & queries
│   └── src/
│       ├── db/               # Connection, pagination, website operations
│       └── models/            # Entity definitions
├── api/                      # actix-web REST API
│   └── src/
│       └── websites/          # Handler, service, DTO
├── docker-compose.yml         # Local Postgres instance
└── .env                       # DATABASE_URL used by both crawler and api
```

## Prerequisites

- Rust (2024 edition toolchain)
- Docker (for the local Postgres database)

## Setup

1. **Start Postgres:**

   ```bash
   docker compose up -d
   ```

2. **Configure the database URL** (already set in `.env`):

   ```
   DATABASE_URL=postgresql://crawler:crawler@localhost:5432/crawler
   ```

3. **Run the crawler:**

   ```bash
   cargo run -p crawler-demo
   ```

   On startup the crawler connects to Postgres, syncs the schema for
   `database::models::*`, then begins crawling from the seed list. Set
   `RUST_LOG` to control log verbosity, e.g.:

   ```bash
   RUST_LOG=info cargo run -p crawler-demo
   ```

4. **Run the API:**

   ```bash
   cargo run -p api
   ```

   The API listens on `127.0.0.1:8080`. Fetch crawled websites with:

   ```bash
   curl "http://127.0.0.1:8080/websites?page=0&page_size=20"
   ```

   `page` and `page_size` are both optional and default to `0` and `20`.

## Notable dependencies

- [`reqwest`](https://docs.rs/reqwest) — HTTP client (with gzip/brotli/deflate
  and `hickory-dns` resolution)
- [`tl`](https://docs.rs/tl) — fast HTML parsing for link extraction
- [`bloomfilter`](https://docs.rs/bloomfilter) — memory-efficient dedup of
  visited paths and hosts
- [`tokio`](https://tokio.rs) — async runtime
- [`sea-orm`](https://www.sea-ql.org/SeaORM/) — async ORM used by both the
  crawler and the API
- [`actix-web`](https://actix.rs) — HTTP server for the API

## Notes / limitations

- The crawler only follows links within the `.dk` TLD.
- Each domain crawl is capped at 50,000 pages and stops early if the response
  isn't `text/html` or exceeds 1 GB in content length.
- Discovered off-domain hosts are only deduplicated in-memory (via a Bloom
  filter sized for ~1,000,000 hosts), so state does not persist across
  restarts.
