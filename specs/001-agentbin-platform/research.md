# Research: Agentbin Platform (v1)

## Platform Architecture Decisions

### Decision 1: Web Framework — Axum

**Decision**: Use Axum as the HTTP framework for `agentbin-server`.

**Rationale**: Axum is built by the tokio team, providing native integration with the async runtime mandated by the constitution. Tower middleware gives us composable layers for auth, request ID injection, logging, and CORS without framework lock-in. Type-safe extractors reduce boilerplate and catch errors at compile time.

**Key features used**:
- `axum::Router` for route composition
- `axum::extract::{Multipart, Path, Query, State, Json}` for request parsing
- `tower_http` middleware: `TraceLayer`, `CorsLayer`, `RequestIdLayer`, `SetRequestIdLayer`
- Graceful shutdown via `axum::serve::with_graceful_shutdown`

**Alternatives considered**:
- Actix Web — fast, mature, but has its own actor system and doesn't compose with tower middleware
- Rocket — ergonomic API, but smaller ecosystem and later to async support

### Decision 2: CLI Framework — clap (derive API)

**Decision**: Use `clap` with the derive macro API for the CLI.

**Rationale**: Standard Rust CLI framework. Derive API gives compile-time checked argument definitions. Subcommands map naturally to the CLI's feature set (upload, list, delete, keygen, admin). Built-in support for `--json` flag via custom output formatting.

**Key features**:
- Subcommand enum for each CLI operation
- `#[arg]` attributes for flags and options
- `--json` global flag for structured output
- Environment variable fallbacks for server URL, key file path

### Decision 3: Markdown Rendering — comrak

**Decision**: Use `comrak` for Markdown-to-HTML conversion.

**Rationale**: `comrak` implements the GitHub Flavored Markdown (GFM) spec, which is the most common markdown dialect used by AI agents. Supports tables, strikethrough, autolinks, task lists, and footnotes. Performance is excellent (C-compatible parser ported to Rust).

**Alternative considered**:
- `pulldown-cmark` — pure Rust, CommonMark only. Lacks GFM table support which agents commonly generate.

### Decision 4: Syntax Highlighting — syntect

**Decision**: Use `syntect` for syntax highlighting of code files (JSON, TOML, YAML, XML, etc.).

**Rationale**: `syntect` uses Sublime Text syntax definitions, supporting 100+ languages out of the box. Can generate HTML with inline styles (no external CSS dependency). Well-maintained and widely used in Rust tools (bat, delta, zola).

**Configuration**: Use a light theme suitable for web viewing (e.g., `InspiredGitHub` or a custom theme). Generate `<pre>` blocks with inline `style` attributes for zero-dependency rendering.

### Decision 5: UID Generation — nanoid (alphanumeric, 10 chars)

**Decision**: Generate UIDs using `nanoid` with a custom alphanumeric alphabet (a-z, A-Z, 0-9) and 10-character length.

**Rationale**: 10 alphanumeric characters give 62^10 ≈ 8.4 × 10^17 combinations — more than sufficient for a single-instance service. URLs like `agentbin.dev/Xn4f8BqR2m` are short and clean. The alphanumeric-only alphabet avoids URL-encoding issues.

**Trade-off**: Not time-sortable. The spec says "time-sortable identifiers are preferred" but not required. Sorting by upload timestamp (stored in metadata) achieves the same result. Constitution principle IV (Ship Simply) favors the simpler approach.

**Alternatives considered**:
- ULID — time-sortable but 26 characters (too long for "short" UIDs)
- Custom base62-encoded timestamp — time-sortable + short, but adds implementation complexity
- UUID v7 — time-sortable but 36 characters with hyphens

### Decision 6: Ed25519 — ed25519-dalek + rand

**Decision**: Use `ed25519-dalek` for key generation, signing, and verification. Use `rand` for cryptographic randomness.

**Rationale**: Already decided in spec clarifications. ed25519-dalek is the standard Rust Ed25519 implementation — fast, well-audited, simple API. Keys are compact (32 bytes public, 64 bytes secret) and signatures are 64 bytes.

**Key storage**: Private key stored as base64-encoded file at `~/.config/agentbin/key.pem` (or configured path). Public key shared with admin for registration.

### Decision 7: HTML Templating — askama

**Decision**: Use `askama` for server-side HTML template rendering.

**Rationale**: Compile-time template validation catches errors before runtime. Jinja2-like syntax is familiar. Templates are type-checked against Rust structs, preventing missing variable errors. Zero runtime overhead.

**Templates needed**:
- `rendered.html` — wrapper for markdown/syntax-highlighted content (includes info badge, version banner)
- `plain.html` — minimal wrapper for plain text files
- `collection.html` — collection overview page with timeline
- `error.html` — error pages (404, 413, etc.)
- `badge.js` — info badge WebComponent (served as static asset)

### Decision 8: Logging — tracing + tracing-subscriber

**Decision**: Use `tracing` for structured logging with `tracing-subscriber` for output formatting.

**Rationale**: `tracing` is the standard async-aware logging framework in the tokio ecosystem. Integrates directly with `tower_http::TraceLayer` for automatic request/response logging. Supports JSON output for production and human-readable output for development, matching constitution principle V.

**Configuration**:
- `RUST_LOG` env var for level control (per constitution: env-based config)
- JSON format in production (`tracing-subscriber::fmt::json`)
- Pretty format in development
- Key events logged: uploads, deletions, auth failures, expiration sweeps, startup, shutdown (per FR-051)

### Decision 9: HTTP Client (CLI) — reqwest

**Decision**: Use `reqwest` for HTTP communication from CLI to server.

**Rationale**: Standard async HTTP client for Rust. Supports multipart uploads, custom headers (for auth signing), and JSON responses. Integrates with tokio.

### Decision 10: Serialization — serde + serde_json

**Decision**: Use `serde` with `serde_json` for all serialization/deserialization.

**Rationale**: Standard. JSON for metadata sidecars, API request/response bodies, CLI `--json` output, and user configuration. `serde` derive macros minimize boilerplate.

### Decision 11: Time — chrono

**Decision**: Use `chrono` for timestamp handling.

**Rationale**: Widely used, supports ISO 8601 formatting (for JSON), timezone-aware types, and duration calculations (for expiry TTL computation). Serde integration via `chrono::serde`.

**Alternative considered**:
- `time` crate — lighter weight, but `chrono` has better serde integration and is more commonly used in web frameworks.

### Decision 12: Configuration — Environment Variables (direct)

**Decision**: Parse server configuration directly from environment variables using `std::env` with helper functions. No configuration framework.

**Rationale**: Constitution principle IV (Ship Simply). The server has only a few config values: `AGENTBIN_STORAGE_PATH`, `AGENTBIN_LISTEN_ADDR`, `AGENTBIN_LOG_FORMAT` (json/pretty), `RUST_LOG`. A full config framework (figment, config-rs) is overkill.

**CLI config precedence** (per FR-047):
1. CLI flags (highest)
2. Environment variables (`AGENTBIN_SERVER_URL`, `AGENTBIN_KEY_FILE`)
3. Config file (`~/.config/agentbin/config.toml`)
4. Defaults (lowest)

`clap` handles this natively with `env` attribute on args.

### Decision 13: Info Badge — WebComponent with Shadow DOM

**Decision**: Implement the info badge as a custom HTML element (`<agentbin-badge>`) using Shadow DOM for style isolation.

**Rationale**: Shadow DOM provides complete CSS and DOM isolation from uploaded HTML content (per FR-011). The badge is a small JavaScript file served as a static asset and injected as a `<script>` tag into every rendered page. The WebComponent renders a floating button that expands into a metadata popover on click.

**Implementation**:
- Badge script: `/_static/badge.js` (served by the server)
- Injected at serve time: `<script src="/_static/badge.js"></script>` + `<agentbin-badge data-meta='...'></agentbin-badge>` appended to HTML
- For non-HTML content (markdown, syntax-highlighted): included in the HTML wrapper template
- For uploaded HTML: injected before `</body>` (or appended to end if no `</body>` tag)
- Metadata passed as JSON in a `data-meta` attribute

### Decision 14: File Storage Layout

**Decision**: Store uploads as files on disk organized by UID and version, with JSON metadata sidecars.

**Layout**:
```
{AGENTBIN_STORAGE_PATH}/
├── uploads/
│   └── {uid}/
│       ├── upload.json          # UID-level metadata (owner, collection, latest_version)
│       └── v{n}/
│           ├── content{.ext}    # Original uploaded file
│           └── meta.json        # Version metadata (timestamps, metadata fields, expiry)
├── collections/
│   └── {name}.json              # Collection membership list
└── users.json                   # Authorized users and public keys
```

**Rationale**: Per FR-039 (no database, file storage with JSON sidecars). Version directories enable atomic version creation (write to temp dir, rename into place per FR-042). UID directories keep related files together.

**Atomic writes** (per FR-042): All file writes use write-to-temp-then-rename pattern. Temp files are written to a `.tmp/` directory within the storage path, then atomically renamed to their final location.

### Decision 15: Content Security

**Decision**: Serve uploaded HTML with restrictive CSP headers. Do not sanitize uploads.

**Rationale**: Per spec, uploaded HTML is agent-generated and semi-trusted. The platform does not sanitize uploads (FR-048 notes) but uses Content Security Policy headers to restrict what uploaded content can execute.

**Headers for uploaded HTML**:
- `Content-Security-Policy: default-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; frame-ancestors 'none'`
- `X-Content-Type-Options: nosniff`
- `X-Frame-Options: DENY`
- `Referrer-Policy: strict-origin-when-cross-origin`

**Note**: `'unsafe-inline'` is necessary because uploaded HTML may contain inline scripts and styles. The CSP still prevents loading external scripts, which limits the attack surface.

### Decision 16: Graceful Shutdown

**Decision**: Use tokio's signal handling with axum's graceful shutdown support.

**Implementation**: Listen for SIGTERM/SIGINT, stop accepting new connections, wait for in-flight requests to complete (with a configurable timeout, default 30 seconds), then exit. Per FR-041.

---

## Release Automation Decisions

### Decision 17: Release Workflow Architecture

**Decision**: Two-tool pipeline — release-plz manages versioning/publishing, cargo-dist manages binary builds/distribution.

**Rationale**: These tools are complementary and communicate through git tags. release-plz handles the "what version" problem (changelog, crates.io, semver), while cargo-dist handles the "distribute binaries" problem (cross-platform builds, installers, Homebrew). This is the standard Rust release pipeline.

**Integration pattern**: release-plz creates git tags → cargo-dist triggers on tags and builds binaries/creates GitHub Releases.

**Configuration**:
- release-plz: `git_release_enable = false` (let cargo-dist own GitHub Releases)
- release-plz: `git_tag_enable = true` (tags are the handoff mechanism)
- cargo-dist: triggers on tag push via `.github/workflows/release.yml`
- A GitHub PAT or App token is needed for release-plz so that tag creation triggers the cargo-dist workflow (default `GITHUB_TOKEN` doesn't trigger downstream workflows)

**Alternatives considered**:
- cargo-release (manual, no PR workflow — rejected for automation)
- release-plz only (no binary distribution — rejected)
- cargo-dist only (no crates.io automation — rejected)

### Decision 18: Tag Naming Convention

**Decision**: Use release-plz workspace defaults — `{{ package }}-v{{ version }}` (e.g., `agentbin-v0.1.0`, `agentbin-server-v0.1.0`).

**Rationale**: cargo-dist recognizes this "Singular Announcement" format natively. Each tag creates one GitHub Release for that specific package. Independent versioning per crate aligns with the constitution's workspace architecture principle.

**Tag → Release mapping**:
- `agentbin-core-v0.1.0` → crates.io publish only (no binaries, library crate)
- `agentbin-v0.1.0` → crates.io publish + GitHub Release with CLI binaries + Homebrew formula
- `agentbin-server-v0.1.0` → crates.io publish + GitHub Release with server binary

### Decision 19: Crate Publishing Strategy

**Decision**: Publish all three crates to crates.io.

| Crate | crates.io name | Published | Binary dist | Homebrew |
|-------|---------------|-----------|-------------|----------|
| core | `agentbin-core` | Yes | No (library) | No |
| cli | `agentbin` | Yes | Yes | Yes |
| server | `agentbin-server` | Yes | Yes | No |

### Decision 20: Target Platforms

**Decision**: Build for 5 targets.

| Target | Runner |
|--------|--------|
| `x86_64-apple-darwin` | `macos-13` |
| `aarch64-apple-darwin` | `macos-14` |
| `x86_64-unknown-linux-gnu` | `ubuntu-22.04` |
| `aarch64-unknown-linux-gnu` | Cross-compilation container |
| `x86_64-pc-windows-msvc` | `windows-2022` |

### Decision 21: Homebrew Distribution

**Decision**: Create `aaronbassett/homebrew-tap` repository. cargo-dist automatically pushes formula updates on CLI releases.

**Install command**: `brew install aaronbassett/tap/agentbin`

Only the CLI is distributed via Homebrew (not the server).

### Decision 22: Installers

| Installer | Platform | Install command |
|-----------|----------|----------------|
| Shell | macOS, Linux | `curl --proto '=https' --tlsv1.2 -LsSf .../agentbin-installer.sh \| sh` |
| PowerShell | Windows | `powershell -c "irm .../agentbin-installer.ps1 \| iex"` |
| Homebrew | macOS, Linux | `brew install aaronbassett/tap/agentbin` |
| cargo-binstall | All | `cargo binstall agentbin` |
| cargo install | All | `cargo install agentbin` |

### Decision 23: GitHub Token Strategy

| Secret | Purpose | Used by |
|--------|---------|------------|
| `CARGO_REGISTRY_TOKEN` | Publish to crates.io | release-plz workflow |
| `HOMEBREW_TAP_TOKEN` | Push formula to tap repo | cargo-dist workflow |
| `RELEASE_PLZ_TOKEN` | Create tags that trigger downstream workflows | release-plz workflow |

### Decision 24: Changelog Configuration

Per-crate `CHANGELOG.md` using git-cliff via release-plz with conventional commit grouping.

### Decision 25: CI Workflow Structure

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | PR, push to main | Build, test, clippy, fmt |
| `release-plz.yml` | Push to main | Version bump PR + crates.io publish |
| `release.yml` | Tag push (`*-v*.*.*`) | Binary builds + GitHub Release + Homebrew |

### Decision 26: Server Distribution

Server binary gets GitHub Release artifacts but NOT Homebrew. Server is deployed to fly.io via Docker.
