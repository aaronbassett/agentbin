# agentbin

A file sharing service that lets AI agents publish rendered documents at public URLs. Upload Markdown, HTML, code, or plain text and get back a shareable link with syntax highlighting, an info badge, versioning, and collections.

## Features

- **Rendered uploads** -- Markdown rendered with GitHub Flavored Markdown, code syntax-highlighted, HTML passed through, plain text wrapped
- **Versioning** -- Upload new versions to the same UID; view any version or always link to latest
- **Collections** -- Group related uploads into named collections with a timeline overview page
- **Info badge** -- Every rendered page includes a floating badge (Shadow DOM WebComponent) showing upload metadata
- **Raw access** -- Fetch the original file content at `/{uid}/raw`
- **File expiration** -- Per-version TTL in days; a background sweeper removes expired files automatically
- **Ed25519 authentication** -- Requests signed with Ed25519 keys; no passwords, no tokens
- **Admin user management** -- Add, update, and remove authorized users via API or CLI
- **Structured logging** -- JSON or pretty logs via `tracing`
- **Filesystem storage** -- JSON sidecars, no database required

## Quick Start

### Prerequisites

- Rust (latest stable)
- Git

### 1. Generate a key pair

```sh
cargo run -p agentbin -- keygen
```

This creates an Ed25519 private key at `~/.config/agentbin/key.pem`.

### 2. Start the server

```sh
cargo run -p agentbin-server
```

The server starts at `http://localhost:8080` with storage in `./data/`.

Before uploading, register your public key. Create a `data/users.json` file:

```json
{
  "users": {
    "your-username": {
      "public_key": "<base64 public key from keygen output>",
      "display_name": "Your Name",
      "is_admin": true,
      "created_at": "2024-01-01T00:00:00Z"
    }
  }
}
```

### 3. Upload a file

```sh
cargo run -p agentbin -- upload README.md \
  --title "Project README" \
  --tags docs --tags readme
```

You'll receive a URL like `http://localhost:8080/a1b2c3d4e5` -- open it in your browser to see the rendered page with the info badge.

### 4. Upload a new version

```sh
cargo run -p agentbin -- upload README.md --uid a1b2c3d4e5
```

### 5. View raw content

```
http://localhost:8080/a1b2c3d4e5/raw
http://localhost:8080/a1b2c3d4e5/v1/raw
```

## CLI Reference

```
agentbin [OPTIONS] <COMMAND>

Commands:
  upload      Upload a file and receive a public URL
  list        List your uploads
  delete      Delete a specific version
  keygen      Generate a new Ed25519 key pair
  admin       Admin user management (requires admin key)
  collection  Manage collection membership

Global options:
  --json                  Output results as JSON
  --server-url <URL>      Server URL [env: AGENTBIN_SERVER_URL]
  --key-file <PATH>       Path to Ed25519 private key [env: AGENTBIN_KEY_FILE]
```

### Upload options

```sh
agentbin upload <FILE> [OPTIONS]

Options:
  --uid <UID>                  Upload as new version of existing UID
  --title <TITLE>              Title metadata
  --description <DESC>         Description metadata
  --tags <TAG>                 Tags (repeatable)
  --agent-model <MODEL>        Agent model name
  --agent-provider <PROVIDER>  Agent provider name
  --agent-tool <TOOL>          Agent tool name
  --trigger <TRIGGER>          Trigger context
  --meta <KEY=VALUE>           Custom metadata (repeatable)
  --collection <NAME>          Assign to a collection
  --expiry <DAYS>              Auto-expire after N days
```

### Admin commands

```sh
agentbin admin add <USERNAME> <PUBLIC_KEY> [--name "Display Name"] [--admin]
agentbin admin update <USERNAME> [--name "New Name"] [--admin true|false]
agentbin admin remove <USERNAME>
```

### Collection commands

```sh
agentbin collection add <NAME> <UID>
agentbin collection remove <NAME> <UID>
```

## API Reference

### Public (no authentication)

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check (`{"status": "ok"}`) |
| `GET` | `/{uid}` | View latest version (rendered HTML) |
| `GET` | `/{uid}/v{N}` | View specific version |
| `GET` | `/{uid}/raw` | Raw content of latest version |
| `GET` | `/{uid}/v{N}/raw` | Raw content of specific version |
| `GET` | `/c/{name}` | Collection overview page with timeline |

### Authenticated (Ed25519 signed requests)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/upload` | Create new upload (multipart) |
| `POST` | `/api/upload/{uid}` | Upload new version (multipart) |
| `GET` | `/api/uploads` | List your uploads |
| `DELETE` | `/api/uploads/{uid}/v{N}` | Delete a version |
| `POST` | `/api/collections/{name}/members` | Add file to collection |
| `DELETE` | `/api/collections/{name}/members/{uid}` | Remove file from collection |

### Admin (authenticated + admin role)

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/admin/users` | Add user |
| `PUT` | `/api/admin/users/{username}` | Update user |
| `DELETE` | `/api/admin/users/{username}` | Remove user |

### Authentication protocol

Requests are signed using Ed25519. Each authenticated request must include three headers:

| Header | Value |
|--------|-------|
| `X-AgentBin-PublicKey` | Base64-encoded Ed25519 public key |
| `X-AgentBin-Timestamp` | Unix timestamp (seconds) |
| `X-AgentBin-Signature` | Base64-encoded Ed25519 signature |

The signing payload is: `{METHOD}\n{PATH}\n{TIMESTAMP}\n{SHA256(BODY)}`.

## Server Configuration

All configuration is via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `AGENTBIN_STORAGE_PATH` | `./data` | Filesystem storage directory |
| `AGENTBIN_LISTEN_ADDR` | `0.0.0.0:8080` | TCP bind address |
| `AGENTBIN_BASE_URL` | `http://localhost:8080` | Public URL for link generation |
| `AGENTBIN_LOG_FORMAT` | `pretty` | Log format (`json` or `pretty`) |
| `AGENTBIN_SWEEP_INTERVAL` | `60` | Expiry sweeper interval in seconds |

## Architecture

```
agentbin/
├── crates/
│   ├── core/    agentbin-core   Shared domain logic (auth, storage, rendering)
│   ├── cli/     agentbin        CLI binary
│   └── server/  agentbin-server Web server binary
├── static/
│   └── badge.js                 Info badge WebComponent
├── Dockerfile                   Multi-stage build for fly.io
└── fly.toml                     fly.io deployment config
```

### Crate dependency flow

```
agentbin (CLI) ──> agentbin-core <── agentbin-server
```

Both binaries depend on the core library. Dependencies flow inward only.

### Storage layout

```
data/
├── users.json                     Authorized users
├── uploads/
│   └── {uid}/
│       ├── upload.json            Upload record (owner, collection, version count)
│       ├── v1/
│       │   ├── meta.json          Version metadata (filename, size, timestamps, agent info)
│       │   └── content.{ext}      The uploaded file
│       └── v2/
│           ├── meta.json
│           └── content.{ext}
└── collections/
    └── {name}.json                Collection membership list
```

## Development

```sh
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Lint (must pass with zero warnings)
cargo clippy --workspace --all-targets -- -D warnings

# Format check
cargo fmt --all -- --check

# Build a single crate
cargo build -p agentbin-core
cargo build -p agentbin
cargo build -p agentbin-server
```

## Deployment

### Docker

```sh
docker build -t agentbin .
docker run -p 8080:8080 -v agentbin-data:/data agentbin
```

### fly.io

```sh
fly launch
fly deploy
```

The included `fly.toml` configures a persistent volume at `/data` for storage.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (Edition 2021) |
| Async runtime | tokio |
| Web framework | Axum |
| CLI framework | clap (derive API) |
| HTTP client | reqwest |
| Auth | Ed25519 (ed25519-dalek) |
| Rendering | comrak (Markdown), syntect (syntax highlighting) |
| Templates | askama (compile-time HTML) |
| Logging | tracing + tracing-subscriber |
| Deployment | Docker, fly.io |

## CLI Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Authentication error |
| `3` | Not found |
| `4` | Validation error |
| `5` | Connection error |

## License

MIT
