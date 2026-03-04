# Changelog

All notable changes to this project will be documented in this file.

## [0.1.2] - 2026-03-04

### Features

- *(agentbin-server)* Include version in health check response



## [0.1.1] - 2026-03-04

### Miscellaneous

- Release v0.1.0



## [0.1.0] - 2026-03-04

### Features

- *(server)* Add entry point with config, state, and graceful shutdown
- *(server)* Add upload endpoint
- *(server)* Add rendered and plain HTML templates and rendering pipeline
- *(server)* Add info badge WebComponent with Shadow DOM isolation
- *(server)* Add view route with security headers and static serving
- *(server)* Add raw content access routes
- *(server)* Add versioned upload endpoint
- Add upload management routes and CLI commands
- Add admin user management routes and CLI commands
- *(cli)* Add metadata flags to upload command
- *(server)* Add background file expiration sweeper
- Add collections with overview page and timeline scrubber

### Bug Fixes

- Critical auth bugs found in code review
- Path traversal, error extraction, and input validation
- *(server)* Use Axum 0.8 route parameter syntax
- *(server)* Use OriginalUri for signature verification path
- *(server)* Return 404 instead of 500 for invalid UIDs
- *(server)* Relax CSP to allow inline scripts and external resources
- *(server)* Move badge.js into server crate for cargo publish

### Miscellaneous

- Scaffold cargo workspace with three crates
- Add release-plz, cliff, and cargo-dist configs
- Set up cargo-dist release workflow and crate metadata


