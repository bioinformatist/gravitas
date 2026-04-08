# Contributing to gravitas

## Prerequisites

- **Rust** (stable, edition 2024)
- **trunk** (`cargo install trunk`) — for WASM UI builds
- **Shuttle CLI** (`cargo install shuttle`) — for local server dev

## Quick Start (zero config)

No API keys needed — mock data is used automatically:

```bash
# ASCII GEX wall
cargo run -p gravitas-cli -- SPY

# Table format
cargo run -p gravitas-cli -- SPY -f table

# JSON output (for scripts / eww bar)
cargo run -p gravitas-cli -- SPY -f json
```

## Data Sources

### Tradier (15-min delayed, free)

1. Register a free brokerage account at [tradier.com](https://tradier.com)
2. Get your API token from the dashboard

```bash
TRADIER_TOKEN=xxx cargo run -p gravitas-cli -- SPY
```

### Futu OpenD (realtime, requires Lv2 subscription)

1. Install [OpenD](https://openapi.futunn.com) standalone program
2. Start OpenD (default: `127.0.0.1:11111`)
3. Compile with the `futu` feature flag:

```bash
FUTU_OPEND_HOST=127.0.0.1 cargo run -p gravitas-cli --features futu -- SPY
```

### Config file (optional)

Create `~/.config/gravitas/config.toml`:

```toml
# Credentials auto-detected. Priority: Futu > Tradier > API.
futu_host = "127.0.0.1"
futu_port = 11111
tradier_token = "your-tradier-token"
api_key = "your-gravitas-api-key"
api_base = "https://your-app.shuttleapp.rs"
```

## Feature Flags

| Flag | Effect |
|------|--------|
| `futu` | Enables Futu OpenD protocol support (adds `prost` + `sha1` deps) |

Default builds have no optional features enabled.

## Local Development

```bash
# Run all tests
cargo test --workspace

# Server (local, with Shuttle CLI)
cd crates/gravitas-server && cargo shuttle run

# WASM UI (requires trunk)
cd crates/gravitas-ui
GRAVITAS_API_URL=http://localhost:8000 trunk serve --open

# Check futu feature compiles
cargo check -p gravitas-fetch --features futu
```

## Legal

**Futu OpenAPI** is governed by a non-commercial license. The `futu` feature is for **personal testing only** — never compile it into release builds or use Futu data in the hosted server/API. See the [Futu API license](https://openapi.futunn.com) for details.
