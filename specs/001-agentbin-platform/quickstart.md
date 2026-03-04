# Quickstart: Agentbin Platform Development

## Prerequisites

- Rust toolchain (latest stable): `rustup update stable`
- `cargo-dist` CLI: `cargo install cargo-dist`
- GitHub repository with the agentbin Cargo workspace set up
- Access to create GitHub secrets on the repository

## Development Commands

```sh
# Build all crates
cargo build --workspace

# Test all crates
cargo test --workspace

# Lint (must pass with zero warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check

# Format fix
cargo fmt --all

# Build a single crate
cargo build -p agentbin-core
cargo build -p agentbin
cargo build -p agentbin-server

# Test a single crate
cargo test -p agentbin-core

# Run the server locally
AGENTBIN_STORAGE_PATH=./data AGENTBIN_LISTEN_ADDR=127.0.0.1:3000 cargo run -p agentbin-server

# Run the CLI
cargo run -p agentbin -- --help
cargo run -p agentbin -- keygen
cargo run -p agentbin -- upload ./test-file.html

# Preview release artifacts
dist plan

# Local build of distributable artifacts
dist build
```

## Core Library Dependencies

These are the primary crates used in `agentbin-core`:

| Crate | Purpose |
|-------|---------|
| `thiserror` | Error type derivation |
| `serde` + `serde_json` | Serialization |
| `chrono` | Timestamp handling |
| `nanoid` | UID generation |
| `ed25519-dalek` | Ed25519 crypto |
| `rand` | Cryptographic randomness |
| `comrak` | Markdown → HTML |
| `syntect` | Syntax highlighting |

## Server Dependencies

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP framework |
| `tokio` | Async runtime |
| `tower-http` | Middleware (tracing, CORS, request ID) |
| `askama` | HTML templates |
| `tracing` + `tracing-subscriber` | Structured logging |
| `anyhow` | Error handling |

## CLI Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | Argument parsing |
| `reqwest` | HTTP client |
| `tokio` | Async runtime |
| `anyhow` | Error handling |

## Environment Variables

### Server

| Variable | Default | Description |
|----------|---------|-------------|
| `AGENTBIN_STORAGE_PATH` | `./data` | Path to persistent storage directory |
| `AGENTBIN_LISTEN_ADDR` | `0.0.0.0:8080` | Listen address |
| `AGENTBIN_LOG_FORMAT` | `pretty` | Log format: `json` or `pretty` |
| `AGENTBIN_BASE_URL` | (required) | Public base URL for generated links |
| `RUST_LOG` | `info` | Log level filter |

### CLI

| Variable | Default | Description |
|----------|---------|-------------|
| `AGENTBIN_SERVER_URL` | `https://agentbin.dev` | Server URL |
| `AGENTBIN_KEY_FILE` | `~/.config/agentbin/key.pem` | Path to private key |

CLI config precedence: flags > env vars > config file (`~/.config/agentbin/config.toml`) > defaults.

## Testing Scenarios

### US1: Upload and Render

```sh
# Upload an HTML file
echo '<h1>Hello</h1>' > /tmp/test.html
cargo run -p agentbin -- upload /tmp/test.html
# → Returns URL, open in browser to verify rendered page

# Upload a Markdown file
echo '# Hello World' > /tmp/test.md
cargo run -p agentbin -- upload /tmp/test.md
# → Verify markdown renders as HTML

# Upload a JSON file
echo '{"key": "value"}' > /tmp/test.json
cargo run -p agentbin -- upload /tmp/test.json
# → Verify syntax-highlighted display
```

### US4: Authentication

```sh
# Generate key pair
cargo run -p agentbin -- keygen
# → Prints public key to share with admin

# Upload without valid key (should fail)
cargo run -p agentbin -- upload /tmp/test.html --key /dev/null
# → Should return auth error
```

### US2: Versioning

```sh
# Upload, get UID
cargo run -p agentbin -- upload /tmp/test.html
# → Note the UID

# Upload new version
cargo run -p agentbin -- upload /tmp/test-v2.html --uid <UID>
# → Version 2 created, both versions accessible
```

### US9: Upload Management

```sh
# List uploads
cargo run -p agentbin -- list
cargo run -p agentbin -- list --json

# Delete a version
cargo run -p agentbin -- delete <UID> --version 1
```

## Release Automation Setup

### One-Time Setup

1. **GitHub Secrets** — add to `aaronbassett/agentbin`:

   | Secret | Source |
   |--------|--------|
   | `CARGO_REGISTRY_TOKEN` | [crates.io/settings/tokens](https://crates.io/settings/tokens) |
   | `RELEASE_PLZ_TOKEN` | GitHub PAT with `contents:write` scope |
   | `HOMEBREW_TAP_TOKEN` | GitHub PAT with `repo` scope |

2. **Homebrew Tap** — create `aaronbassett/homebrew-tap` on GitHub with just a README.

3. **Initialize cargo-dist** — run `dist init` to generate `.github/workflows/release.yml`.

### Release Flow

1. Write code, commit with conventional commits (`feat:`, `fix:`, etc.)
2. Push/merge to `main`
3. release-plz opens a Release PR with version bumps + changelogs
4. Review and merge the Release PR
5. release-plz publishes to crates.io and creates git tags
6. Tags trigger cargo-dist → builds binaries → creates GitHub Release → pushes Homebrew formula

### User Install Commands

```sh
# Homebrew (macOS/Linux)
brew install aaronbassett/tap/agentbin

# Shell installer (macOS/Linux)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/aaronbassett/agentbin/releases/latest/download/agentbin-installer.sh | sh

# PowerShell (Windows)
powershell -c "irm https://github.com/aaronbassett/agentbin/releases/latest/download/agentbin-installer.ps1 | iex"

# cargo install (any platform with Rust)
cargo install agentbin

# cargo binstall (pre-built, auto-detected)
cargo binstall agentbin
```

## Deployment (fly.io)

```sh
# Deploy server
fly deploy

# Set secrets
fly secrets set AGENTBIN_STORAGE_PATH=/data
fly secrets set AGENTBIN_BASE_URL=https://agentbin.dev
fly secrets set AGENTBIN_LOG_FORMAT=json

# Check health
curl https://agentbin.dev/health
```
