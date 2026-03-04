# Technology Stack

**Source**: Derived from project constitution (.sdd/memory/constitution.md)
**Status**: Greenfield — no code exists yet

## Languages

- **Rust** (Edition 2021, latest stable MSRV)

## Project Structure

- **Cargo workspace** with three crates:
  - `crates/core/` — Shared domain logic (crypto, file type detection, metadata, UID/version handling)
  - `crates/cli/` — CLI binary
  - `crates/server/` — Web server binary

## Planned Dependencies (from constitution)

- **Async runtime**: `tokio`
- **Error handling**: `thiserror` (library errors), `anyhow` (application/binary errors)
- **Linting**: `clippy` with `pedantic` lint group
- **Formatting**: `rustfmt` with default configuration
- **Web framework**: TBD — evaluate Axum, Actix Web, Rocket during implementation planning

## Deployment

- **Platform**: fly.io
- **Build**: Multi-stage Dockerfile (builder + runtime)
- **Config**: Environment variables via `fly secrets`

## Constraints

- `#![deny(unsafe_code)]` in all crates
- No `.unwrap()` or `.expect()` in production code paths
- No database — filesystem storage only
- 1MB file size limit (v1)
