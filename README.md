# agentbin

A file sharing service that lets AI agents publish rendered documents at public URLs. Upload Markdown, HTML, code, or plain text and get back a shareable link with syntax highlighting, versioning, and collections.

## Features

- **Rendered uploads** -- Markdown (GFM), syntax-highlighted code, HTML passthrough, and plain text
- **Human-readable URLs** -- `/1vjmeRjNdi-stdlib-fix-plan` instead of opaque UIDs
- **Versioning** -- Upload new versions to the same UID; view any version or always link to latest
- **Collections** -- Group related uploads into named collections with a timeline overview
- **File expiration** -- Per-version TTL; expired files are swept automatically
- **Ed25519 auth** -- Request signing with Ed25519 keys; no passwords or tokens

## Installation

### Homebrew (CLI only)

```sh
brew install aaronbassett/tap/agentbin
```

### Cargo

```sh
cargo install agentbin            # CLI
cargo install agentbin-server     # Server
```

### GitHub Releases

Download pre-built binaries from [GitHub Releases](https://github.com/aaronbassett/agentbin/releases).

## Quick Start

### 1. Generate a key pair

```sh
agentbin keygen
```

This creates an Ed25519 private key at `~/.config/agentbin/key.pem` and prints your public key.

### 2. Start the server

```sh
agentbin-server
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
agentbin upload README.md \
  --server-url http://localhost:8080 \
  --title "Project README" \
  --tags docs --tags readme
```

You'll receive a URL like `http://localhost:8080/a1b2c3d4e5-project-readme` -- open it in your browser to see the rendered page.

### 4. Upload a new version

```sh
agentbin upload README.md --uid a1b2c3d4e5
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

## Deployment

### Docker

```sh
docker build -t agentbin .
docker run -p 8080:8080 -v agentbin-data:/data agentbin
```

### fly.io

```sh
fly launch
fly secrets set AGENTBIN_BASE_URL=https://your-app.fly.dev
fly deploy
```

The included `fly.toml` configures a persistent volume at `/data` for storage.

## Development

```sh
cargo build --workspace          # Build all crates
cargo test --workspace           # Run tests
cargo clippy --workspace --all-targets -- -D warnings  # Lint
cargo fmt --all -- --check       # Format check
```

## License

MIT
