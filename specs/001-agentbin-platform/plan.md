# Implementation Plan: Agentbin Platform (v1)

**Branch**: `001-agentbin-platform` | **Date**: 2026-03-04 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-agentbin-platform/spec.md`
**Scope**: Full platform — workspace scaffold, core library, CLI binary, web server, CI/CD release automation

## Summary

Build the agentbin platform: a file sharing service that lets AI agents publish rendered documents at public URLs. The system is a Cargo workspace with three crates: a shared core library (domain logic, storage, auth, rendering), a CLI binary (upload, manage, keygen), and a web server binary (HTTP API, rendering pipeline, info badge). Storage is filesystem-based with JSON metadata sidecars. Authentication uses Ed25519 key signing. The CI/CD pipeline uses release-plz for versioning/publishing and cargo-dist for cross-platform binary builds.

## Technical Context

**Language/Version**: Rust (Edition 2021, latest stable MSRV)
**Async Runtime**: tokio
**Web Framework**: Axum (see research.md Decision 1)
**CLI Framework**: clap with derive API (see research.md Decision 2)
**Error Handling**: `thiserror` (core library), `anyhow` (cli + server binaries)
**Serialization**: serde + serde_json
**HTTP Client**: reqwest (CLI → server communication)
**Auth**: ed25519-dalek + rand (Ed25519 key pairs and request signing)
**Markdown Rendering**: comrak (GFM-compatible)
**Syntax Highlighting**: syntect (Sublime Text syntax definitions)
**HTML Templating**: askama (compile-time validated templates)
**Logging**: tracing + tracing-subscriber (structured, async-aware)
**Time**: chrono (timestamps, expiry calculations)
**UID Generation**: nanoid (10-char alphanumeric)
**Storage**: Filesystem with JSON sidecars (no database)
**Deployment**: fly.io (server), GitHub Releases + Homebrew (CLI)
**Release Tooling**: release-plz + cargo-dist (CI, not runtime deps)
**Target Platforms**: Linux (amd64, aarch64), macOS (amd64, aarch64), Windows (amd64)
**Performance Goals**: Upload-to-URL under 5 seconds (SC-001)
**Constraints**:
- `#![deny(unsafe_code)]` in all crates
- No `.unwrap()` / `.expect()` in production paths
- 1MB file size limit per upload
- Single server instance with persistent storage volume
- Conventional commits required

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Workspace Architecture | PASS | Three crates (core, cli, server). Dependencies flow inward. Each compiles/tests independently. |
| II. Pragmatic Testing | PASS | Core business logic (auth, storage, rendering, UID) gets unit tests. Server endpoints get integration tests. CLI gets argument parsing tests. Trivial code skipped. |
| III. Safe by Default | PASS | `deny(unsafe_code)` everywhere. clippy pedantic. thiserror/anyhow for errors. Input validation at HTTP and CLI boundaries. No secrets in logs. |
| IV. Ship Simply | PASS | Filesystem storage (no database). Standard crate ecosystem (axum, clap, serde). No premature abstractions. Features scoped to spec requirements only. |
| V. Deployable Always | PASS | CI gates merges. `/health` endpoint. Structured logging. fly.io deployment via Dockerfile. Env-var configuration. |

**Gate result**: PASS — no violations.

## Project Structure

### Documentation (this feature)

```text
specs/001-agentbin-platform/
├── plan.md              # This file
├── research.md          # All technical decisions
├── data-model.md        # Entities, storage layout, schemas
├── quickstart.md        # Developer setup guide
└── contracts/           # API contracts, auth protocol, CI/CD configs
```

### Source Code (repository root)

```text
agentbin/
├── Cargo.toml                        # Workspace root
├── crates/
│   ├── core/
│   │   ├── Cargo.toml                # agentbin-core (library)
│   │   └── src/
│   │       ├── lib.rs                # Public API, deny(unsafe_code)
│   │       ├── error.rs              # Error types (thiserror)
│   │       ├── uid.rs                # UID generation (nanoid)
│   │       ├── metadata.rs           # Metadata types (well-known + arbitrary)
│   │       ├── filetype.rs           # File type detection by extension
│   │       ├── auth.rs               # Ed25519 keygen, signing, verification
│   │       ├── storage.rs            # Storage trait + filesystem implementation
│   │       └── render.rs             # Markdown→HTML, syntax highlighting
│   ├── cli/
│   │   ├── Cargo.toml                # agentbin (binary)
│   │   ├── dist.toml                 # cargo-dist: shell + powershell + homebrew
│   │   └── src/
│   │       ├── main.rs               # Entry point, deny(unsafe_code)
│   │       ├── commands/
│   │       │   ├── mod.rs            # Subcommand enum
│   │       │   ├── upload.rs         # Upload file / new version
│   │       │   ├── list.rs           # List user's uploads
│   │       │   ├── delete.rs         # Delete specific version
│   │       │   ├── keygen.rs         # Generate Ed25519 key pair
│   │       │   └── admin.rs          # User management (add/remove/update)
│   │       ├── config.rs             # CLI config (flags > env > file > defaults)
│   │       ├── output.rs             # Human/JSON output formatting
│   │       └── signing.rs            # Request signing (HTTP headers)
│   └── server/
│       ├── Cargo.toml                # agentbin-server (binary)
│       ├── dist.toml                 # cargo-dist: shell only
│       └── src/
│           ├── main.rs               # Entry point, deny(unsafe_code), graceful shutdown
│           ├── state.rs              # App state (storage path, user config)
│           ├── config.rs             # Server config from env vars
│           ├── routes/
│           │   ├── mod.rs            # Router composition
│           │   ├── upload.rs         # POST /api/upload, POST /api/upload/{uid}
│           │   ├── view.rs           # GET /{uid}, GET /{uid}/v{n} (rendered)
│           │   ├── raw.rs            # GET /{uid}/raw, GET /{uid}/v{n}/raw
│           │   ├── manage.rs         # GET /api/uploads, DELETE /api/uploads/{uid}/v{n}
│           │   ├── admin.rs          # POST/PUT/DELETE /api/admin/users/{username}
│           │   ├── collection.rs     # GET /c/{name}, collection member management
│           │   └── health.rs         # GET /health
│           ├── middleware/
│           │   ├── mod.rs
│           │   ├── auth.rs           # Ed25519 signature verification + replay protection
│           │   └── request_id.rs     # X-Request-Id header (echo or generate)
│           ├── badge.rs              # Info badge injection (WebComponent script + data)
│           ├── render.rs             # Rendering pipeline (dispatch by file type)
│           ├── expiry.rs             # Background expiration sweeper
│           └── templates/
│               ├── rendered.html     # Wrapper for rendered content (info badge, version banner)
│               ├── plain.html        # Minimal plain text wrapper
│               ├── collection.html   # Collection overview with timeline
│               └── error.html        # Error pages
├── static/
│   └── badge.js                      # Info badge WebComponent
├── .github/workflows/
│   ├── ci.yml                        # Build, test, clippy, fmt on PRs
│   ├── release-plz.yml               # Version bumps + crates.io publish on main
│   └── release.yml                   # Binary builds + GitHub Release on tag (generated by dist init)
├── release-plz.toml                  # release-plz config
├── dist-workspace.toml               # cargo-dist config
├── cliff.toml                        # Changelog config
├── Dockerfile                        # Multi-stage build for fly.io
└── fly.toml                          # fly.io deployment config
```

**Structure Decisions**:
- Core library owns all domain logic; binaries are thin wrappers over core
- Server templates live in the server crate (askama compiles them in)
- Static assets (badge.js) served by the server directly
- CI/CD config at workspace root
- Homebrew tap is a separate repo (aaronbassett/homebrew-tap)

## User Story → Crate Mapping

| User Story | Core | CLI | Server |
|------------|------|-----|--------|
| US1: Upload & Render (P1) | filetype, render, storage, uid, metadata | upload command | upload route, view route, render pipeline, badge injection |
| US2: Versioning (P2) | storage (version handling) | upload --uid flag | upload route (version), view route (version banner) |
| US3: Raw Access (P2) | storage (content retrieval) | — | raw route |
| US4: Key-Based Auth (P1) | auth (keygen, signing, verification) | keygen command, signing module | auth middleware |
| US5: Admin User Mgmt (P2) | storage (user config) | admin commands | admin routes |
| US6: Collections (P3) | storage (collection ops), metadata | upload --collection flag | collection routes, collection template |
| US7: Metadata & Badge (P2) | metadata types | upload --meta flags | badge injection, badge.js |
| US8: File Expiration (P3) | metadata (expiry field) | upload --expiry flag | expiry sweeper |
| US9: Upload Management (P2) | storage (list, delete) | list/delete commands | manage routes |

## Post-Design Constitution Re-Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Workspace Architecture | PASS | Core owns domain logic. CLI and server depend on core, never reverse. Each crate compiles independently. |
| II. Pragmatic Testing | PASS | Core: unit tests for auth, storage, rendering, UID, filetype, metadata. Server: integration tests for upload/view/auth/admin. CLI: arg parsing + output format tests. Trivial code skipped. |
| III. Safe by Default | PASS | deny(unsafe_code). clippy pedantic. thiserror/anyhow. Input validation at HTTP boundaries (size limit, file type, signature) and CLI (clap validation). CSP headers on served content. No secrets in logs. |
| IV. Ship Simply | PASS | Filesystem storage, no database. Standard ecosystem crates. No custom abstractions. nanoid for UIDs (not custom encoder). env vars for config (no framework). askama for templates (compile-time, no runtime engine). |
| V. Deployable Always | PASS | CI gates all merges (build, test, clippy, fmt). /health endpoint. Structured JSON logging. fly.io via Dockerfile. Env-var config. Automated releases via release-plz + cargo-dist. |

**Gate result**: PASS — design verified.

## Phase 2: Local Dev Environment

**Status**: Partially configured. CI workflow exists (ci.yml) with build, test, clippy, fmt jobs. Local pre-commit hooks and dev tooling to be set up during implementation.

**Needed**:
- `cargo clippy --workspace --all-targets -- -D warnings` (per constitution III)
- `cargo fmt --all -- --check` (per constitution III)
- `cargo test --workspace` (per constitution II)
- Pre-commit hooks for fmt/clippy (per spec DS-002)
- `dist init` to generate release.yml (per this plan)
- Dockerfile for fly.io deployment (per constitution V)

## Complexity Tracking

No violations — no complexity justifications needed.
