# Tasks: Agentbin Platform (v1)

**Input**: Design documents from `/specs/001-agentbin-platform/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md, contracts/
**Branch**: `001-agentbin-platform`
**Tests**: Not explicitly requested — test tasks omitted. Constitution II requires core business logic tests alongside implementation.

**Organization**: Tasks grouped by user story priority. All Rust files use `devs:rust-dev` agent.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story (US1–US9) or [GIT] for git workflow tasks
- Exact file paths included in all implementation tasks

---

## Phase 1: Setup (Scaffold Cargo Workspace & CI/CD)

**Purpose**: Initialize the workspace, scaffold all three crates, install CI/CD and release configuration. After this phase, `cargo build --workspace` succeeds and CI runs on PRs.

### Phase Start
- [ ] T001 [GIT] Verify on main branch and working tree is clean
- [ ] T002 [GIT] Pull latest changes from origin/main
- [ ] T003 [GIT] Create feature branch: 001-agentbin-platform

### Implementation
- [ ] T004 Create workspace Cargo.toml with members = ["crates/core", "crates/cli", "crates/server"], workspace.package (edition = "2021"), and workspace.dependencies for shared deps (use devs:rust-dev agent)
- [ ] T005 [P] Scaffold crates/core/ with Cargo.toml (name = "agentbin-core", deps: thiserror, serde, serde_json, chrono, nanoid, ed25519-dalek, rand, comrak, syntect, sha2, base64) and src/lib.rs with `#![deny(unsafe_code)]` (use devs:rust-dev agent)
- [ ] T006 [P] Scaffold crates/cli/ with Cargo.toml (name = "agentbin", deps: clap derive, reqwest multipart, tokio, anyhow, serde, serde_json, agentbin-core) and src/main.rs with `#![deny(unsafe_code)]` stub (use devs:rust-dev agent)
- [ ] T007 [P] Scaffold crates/server/ with Cargo.toml (name = "agentbin-server", deps: axum, tokio, tower-http, askama, tracing, tracing-subscriber, anyhow, serde, serde_json, agentbin-core) and src/main.rs with `#![deny(unsafe_code)]` stub (use devs:rust-dev agent)
- [ ] T008 Add .gitignore with Rust defaults (target/, *.swp, .DS_Store) at workspace root
- [ ] T009 [GIT] Commit: chore: scaffold cargo workspace with three crates
- [ ] T010 [P] Deploy release-plz.toml from specs/001-agentbin-platform/contracts/release-plz.toml to workspace root
- [ ] T011 [P] Deploy cliff.toml from specs/001-agentbin-platform/contracts/cliff.toml to workspace root
- [ ] T012 [P] Deploy dist-workspace.toml from specs/001-agentbin-platform/contracts/dist-workspace.toml to workspace root
- [ ] T013 [P] Deploy crates/cli/dist.toml from specs/001-agentbin-platform/contracts/cli-dist.toml
- [ ] T014 [P] Deploy crates/server/dist.toml from specs/001-agentbin-platform/contracts/server-dist.toml
- [ ] T015 [GIT] Commit: chore: add release-plz, cliff, and cargo-dist configs
- [ ] T016 Deploy .github/workflows/ci.yml from specs/001-agentbin-platform/contracts/ci.yml
- [ ] T017 Deploy .github/workflows/release-plz.yml from specs/001-agentbin-platform/contracts/release-plz.yml
- [ ] T018 [GIT] Commit: ci: add CI and release-plz workflows
- [ ] T019 Initialize cargo-dist: run `dist init` to generate .github/workflows/release.yml (use devs:rust-dev agent)
- [ ] T020 [GIT] Commit: ci: add cargo-dist release workflow
- [ ] T021 Verify workspace compiles: `cargo build --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check` (use devs:rust-dev agent)
- [ ] T022 [GIT] Commit: fix any build or lint issues from scaffold

### Phase Completion
- [ ] T023 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T024 [GIT] Create/update PR to main with phase summary
- [ ] T025 [GIT] Verify all CI checks pass
- [ ] T026 [GIT] Report PR ready status

---

## Phase 2: Foundational (Core Library + Server/CLI Skeletons)

**Purpose**: Build all core library modules and server/CLI skeletons that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

### Phase Start
- [ ] T027 [GIT] Verify working tree is clean before starting Phase 2
- [ ] T028 [GIT] Pull and rebase on origin/main if needed
- [ ] T029 Create specs/001-agentbin-platform/retro/P2.md for this phase
- [ ] T030 [GIT] Commit: chore: initialize phase 2 retro

### Core Library — Types & Utilities
- [ ] T031 [P] Implement error types (CoreError enum with thiserror: StorageError, AuthError, RenderError, ValidationError) in crates/core/src/error.rs (use devs:rust-dev agent)
- [ ] T032 [P] Implement UID generation (nanoid, 10-char alphanumeric alphabet per Decision 5) in crates/core/src/uid.rs (use devs:rust-dev agent)
- [ ] T033 [P] Implement file type detection (FileType enum, detect from extension, content_type(), rendering strategy per data-model.md table) in crates/core/src/filetype.rs (use devs:rust-dev agent)
- [ ] T034 Update crates/core/src/lib.rs with pub mod declarations for error, uid, filetype (use devs:rust-dev agent)
- [ ] T035 [GIT] Commit: feat(core): add error types, UID generation, and file type detection

### Core Library — Domain Models & Auth
- [ ] T036 Implement metadata types (Metadata struct with well-known fields + custom HashMap per data-model.md) in crates/core/src/metadata.rs (use devs:rust-dev agent)
- [ ] T037 [GIT] Commit: feat(core): add metadata types
- [ ] T038 Implement Ed25519 auth (keygen via ed25519-dalek + rand, signing payload construction per auth-protocol.md, verification, base64 encoding) in crates/core/src/auth.rs (use devs:rust-dev agent)
- [ ] T039 [GIT] Commit: feat(core): add Ed25519 auth module

### Core Library — Storage & Rendering
- [ ] T040 Implement Storage trait and FileStorage (create_upload, store_version with content + meta.json, get_version, list_uploads, delete_version, user CRUD, collection CRUD, atomic writes via temp-then-rename per FR-042, storage layout per data-model.md) in crates/core/src/storage.rs (use devs:rust-dev agent)
- [ ] T041 [GIT] Commit: feat(core): add filesystem storage with atomic writes
- [ ] T042 Implement rendering (Markdown→HTML via comrak with GFM extensions, syntax highlighting via syntect with inline styles, plain text wrapping) in crates/core/src/render.rs (use devs:rust-dev agent)
- [ ] T043 [GIT] Commit: feat(core): add rendering pipeline (markdown + syntax highlighting)
- [ ] T044 Update crates/core/src/lib.rs with pub mod for metadata, auth, storage, render and re-export public types (use devs:rust-dev agent)
- [ ] T045 [GIT] Commit: feat(core): wire up complete public API

### Server Skeleton
- [ ] T046 [P] Implement server config from env vars (AGENTBIN_STORAGE_PATH, AGENTBIN_LISTEN_ADDR, AGENTBIN_BASE_URL, AGENTBIN_LOG_FORMAT, RUST_LOG per quickstart.md) in crates/server/src/config.rs (use devs:rust-dev agent)
- [ ] T047 [P] Implement AppState struct (storage instance, base URL, user config) in crates/server/src/state.rs (use devs:rust-dev agent)
- [ ] T048 Implement server entry point with tokio runtime, tracing subscriber init, and graceful shutdown (SIGTERM/SIGINT with 30s timeout per Decision 16) in crates/server/src/main.rs (use devs:rust-dev agent)
- [ ] T049 [GIT] Commit: feat(server): add entry point with config, state, and graceful shutdown
- [ ] T050 [P] Implement request ID middleware (echo X-Request-Id or generate UUID per FR-052) in crates/server/src/middleware/request_id.rs (use devs:rust-dev agent)
- [ ] T051 [P] Implement auth middleware (extract X-AgentBin-* headers, validate timestamp ±300s, lookup public key, verify Ed25519 signature, attach user context per auth-protocol.md) in crates/server/src/middleware/auth.rs (use devs:rust-dev agent)
- [ ] T052 [P] Create middleware mod.rs in crates/server/src/middleware/mod.rs (use devs:rust-dev agent)
- [ ] T053 [GIT] Commit: feat(server): add auth and request ID middleware
- [ ] T054 Implement health endpoint (GET /health → {"status":"ok"} per FR-038) in crates/server/src/routes/health.rs (use devs:rust-dev agent)
- [ ] T055 Implement router composition (layer middleware: tracing, request ID globally, auth on /api/* routes, mount health) in crates/server/src/routes/mod.rs (use devs:rust-dev agent)
- [ ] T056 Create error template in crates/server/src/templates/error.html
- [ ] T057 [GIT] Commit: feat(server): add router with health endpoint and error template

### CLI Skeleton
- [ ] T058 Implement CLI entry point with clap derive App, global --json flag, subcommand enum in crates/cli/src/main.rs (use devs:rust-dev agent)
- [ ] T059 [P] Create subcommand enum (Upload, List, Delete, Keygen, Admin) with stub handlers in crates/cli/src/commands/mod.rs (use devs:rust-dev agent)
- [ ] T060 [P] Implement CLI config (server URL, key file path) with precedence: flags > env > config file > defaults per Decision 12 in crates/cli/src/config.rs (use devs:rust-dev agent)
- [ ] T061 [P] Implement output formatting (OutputFormat enum, format_success/format_error/format_list for human + JSON per FR-036/FR-037) in crates/cli/src/output.rs (use devs:rust-dev agent)
- [ ] T062 [P] Implement request signing (construct signing payload per auth-protocol.md, sign with ed25519, attach auth headers to reqwest requests) in crates/cli/src/signing.rs (use devs:rust-dev agent)
- [ ] T063 [GIT] Commit: feat(cli): add entry point with clap, config, output, and signing

### Verification
- [ ] T064 Verify full workspace compiles and core tests pass: `cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` (use devs:rust-dev agent)
- [ ] T065 [GIT] Commit: fix any build/test/lint issues

### Phase End
- [ ] T066 Run /sdd:map incremental for Phase 2 changes
- [ ] T067 [GIT] Commit: docs: update codebase documents for phase 2
- [ ] T068 Review retro/P2.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T069 [GIT] Commit: docs: finalize phase 2 retro
- [ ] T070 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T071 [GIT] Create/update PR to main with phase summary
- [ ] T072 [GIT] Verify all CI checks pass
- [ ] T073 [GIT] Report PR ready status

**Checkpoint**: Core library complete with all modules. Server starts, serves /health, has auth middleware. CLI parses args and can sign requests. All crates compile and pass lint.

---

## Phase 3: US4 — Key-Based Authentication (Priority: P1)

**Goal**: CLI keygen command generates Ed25519 key pair. Server auth middleware verifies signed requests and rejects invalid ones.

**Independent Test**: Generate key pair via `agentbin keygen`, seed users.json with public key, attempt signed request (accepted) and unsigned request (rejected with AUTH_INVALID_SIGNATURE)

**Why first**: Auth is prerequisite for all upload operations

### Phase Start
- [ ] T074 [GIT] Verify working tree is clean before starting Phase 3
- [ ] T075 [GIT] Pull and rebase on origin/main if needed
- [ ] T076 [US4] Create specs/001-agentbin-platform/retro/P3.md for this phase
- [ ] T077 [GIT] Commit: chore: initialize phase 3 retro

### Implementation
- [ ] T078 [US4] Implement keygen command (generate Ed25519 key pair, save private key to ~/.config/agentbin/key.pem with 0600 perms, print public key to stdout per auth-protocol.md Key Generation section) in crates/cli/src/commands/keygen.rs (use devs:rust-dev agent)
- [ ] T079 [GIT] Commit: feat(cli): add keygen command
- [ ] T080 [US4] Verify server starts with auth middleware and /health works: `cargo run -p agentbin-server` (use devs:rust-dev agent)
- [ ] T081 [US4] Verify keygen works end-to-end: `cargo run -p agentbin -- keygen` (use devs:rust-dev agent)
- [ ] T082 [GIT] Commit: fix: resolve any auth integration issues

### Phase End
- [ ] T083 [US4] Run /sdd:map incremental for Phase 3 changes
- [ ] T084 [GIT] Commit: docs: update codebase documents for phase 3
- [ ] T085 [US4] Review retro/P3.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T086 [GIT] Commit: docs: finalize phase 3 retro
- [ ] T087 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T088 [GIT] Create/update PR to main with phase summary
- [ ] T089 [GIT] Verify all CI checks pass
- [ ] T090 [GIT] Report PR ready status

**Checkpoint**: Auth functional — keygen generates keys, server verifies signed requests, rejects unsigned/invalid

---

## Phase 4: US1 — Upload & Render (Priority: P1) MVP

**Goal**: Agent uploads a file via CLI, receives a public URL. Visitors see rendered content: HTML as-is, Markdown as formatted HTML, JSON/TOML/YAML/XML with syntax highlighting, plain text in minimal wrapper. Info badge visible on every page.

**Independent Test**: Upload HTML file → get URL → open in browser → see rendered page with info badge. Repeat with .md, .json, .txt files.

### Phase Start
- [ ] T091 [GIT] Verify working tree is clean before starting Phase 4
- [ ] T092 [GIT] Pull and rebase on origin/main if needed
- [ ] T093 [US1] Create specs/001-agentbin-platform/retro/P4.md for this phase
- [ ] T094 [GIT] Commit: chore: initialize phase 4 retro

### Server — Upload Route
- [ ] T095 [US1] Implement upload route (POST /api/upload: parse multipart file + optional metadata JSON, validate 1MB size limit per FR-007, detect file type, generate UID, store via core storage, return JSON response per api-spec.md) in crates/server/src/routes/upload.rs (use devs:rust-dev agent)
- [ ] T096 [GIT] Commit: feat(server): add upload endpoint

### Server — Templates & Rendering
- [ ] T097 [US1] Create rendered.html askama template (HTML shell wrapping rendered content with CSS, version banner placeholder, badge script injection point) in crates/server/src/templates/rendered.html (use devs:rust-dev agent)
- [ ] T098 [P] [US1] Create plain.html askama template (minimal wrapper preserving plain-text feel) in crates/server/src/templates/plain.html (use devs:rust-dev agent)
- [ ] T099 [GIT] Commit: feat(server): add rendered and plain HTML templates
- [ ] T100 [US1] Implement rendering pipeline (dispatch by file type → HTML passthrough / Markdown render / syntax highlight / plain text wrap, call core render functions) in crates/server/src/render.rs (use devs:rust-dev agent)
- [ ] T101 [GIT] Commit: feat(server): add rendering pipeline

### Server — Info Badge
- [ ] T102 [US1] Implement info badge injection (for HTML: inject script+element before </body>, for rendered content: include in template, serialize metadata to data-meta JSON attribute per Decision 13) in crates/server/src/badge.rs (use devs:rust-dev agent)
- [ ] T103 [US1] Create badge WebComponent (<agentbin-badge> custom element with Shadow DOM, floating button, metadata popover on click per FR-010/FR-011/FR-012) in static/badge.js
- [ ] T104 [GIT] Commit: feat(server): add info badge WebComponent with Shadow DOM isolation

### Server — View Route
- [ ] T105 [US1] Implement view route (GET /{uid}: load latest version, render by file type, inject badge, serve with Content-Type and security headers CSP/X-Content-Type-Options/X-Frame-Options per Decision 15) in crates/server/src/routes/view.rs (use devs:rust-dev agent)
- [ ] T106 [US1] Add static file serving for /_static/badge.js and register upload + view routes in crates/server/src/routes/mod.rs (use devs:rust-dev agent)
- [ ] T107 [GIT] Commit: feat(server): add view route with security headers and static serving

### CLI — Upload Command
- [ ] T108 [US1] Implement upload command (read file, construct multipart request, sign via signing.rs, POST to /api/upload, display URL in human/JSON format per FR-036) in crates/cli/src/commands/upload.rs (use devs:rust-dev agent)
- [ ] T109 [US1] Wire upload and keygen commands into CLI dispatch in crates/cli/src/commands/mod.rs (use devs:rust-dev agent)
- [ ] T110 [GIT] Commit: feat(cli): add upload command

### Verification
- [ ] T111 [US1] End-to-end validation: start server, seed users.json, upload HTML/MD/JSON files via CLI, verify URLs return rendered content with badge (use devs:rust-dev agent)
- [ ] T112 [GIT] Commit: fix: resolve any end-to-end integration issues

### Phase End
- [ ] T113 [US1] Run /sdd:map incremental for Phase 4 changes
- [ ] T114 [GIT] Commit: docs: update codebase documents for phase 4
- [ ] T115 [US1] Review retro/P4.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T116 [GIT] Commit: docs: finalize phase 4 retro
- [ ] T117 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T118 [GIT] Create/update PR to main with phase summary
- [ ] T119 [GIT] Verify all CI checks pass
- [ ] T120 [GIT] Report PR ready status

**Checkpoint**: MVP complete — agent uploads file, receives URL, visitors see rendered content with info badge. Core value proposition working end-to-end.

---

## Phase 5: US2 — Versioning (Priority: P2)

**Goal**: Uploading with existing UID creates new version. Each version has distinct URL. Old versions show "newer version available" banner. Base URL serves latest.

**Independent Test**: Upload file (get UID), upload again with --uid → version 2 created. Access v1 URL → see banner linking to v2. Access base URL → see v2.

### Phase Start
- [ ] T121 [GIT] Verify working tree is clean before starting Phase 5
- [ ] T122 [GIT] Pull and rebase on origin/main if needed
- [ ] T123 [US2] Create specs/001-agentbin-platform/retro/P5.md for this phase
- [ ] T124 [GIT] Commit: chore: initialize phase 5 retro

### Implementation
- [ ] T125 [US2] Add version upload route (POST /api/upload/{uid}: validate ownership, increment version, store per api-spec.md) in crates/server/src/routes/upload.rs (use devs:rust-dev agent)
- [ ] T126 [GIT] Commit: feat(server): add versioned upload route
- [ ] T127 [US2] Extend view route for versioned URLs (GET /{uid}/v{n}), add "newer version available" banner when viewing non-latest version per FR-015 in crates/server/src/routes/view.rs (use devs:rust-dev agent)
- [ ] T128 [US2] Update rendered.html template with conditional version banner in crates/server/src/templates/rendered.html
- [ ] T129 [GIT] Commit: feat(server): add versioned view URLs with version banner
- [ ] T130 [US2] Add --uid flag to CLI upload command for versioned uploads in crates/cli/src/commands/upload.rs (use devs:rust-dev agent)
- [ ] T131 [GIT] Commit: feat(cli): add --uid flag for version uploads

### Phase End
- [ ] T132 [US2] Run /sdd:map incremental for Phase 5 changes
- [ ] T133 [GIT] Commit: docs: update codebase documents for phase 5
- [ ] T134 [US2] Review retro/P5.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T135 [GIT] Commit: docs: finalize phase 5 retro
- [ ] T136 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T137 [GIT] Create/update PR to main with phase summary
- [ ] T138 [GIT] Verify all CI checks pass
- [ ] T139 [GIT] Report PR ready status

**Checkpoint**: Versioning works — multiple versions per UID, distinct URLs, version banner on old versions

---

## Phase 6: US3 — Raw Access (Priority: P2)

**Goal**: Every uploaded file has a parallel raw URL serving original content as text/plain

**Independent Test**: Upload file, fetch /{uid}/raw with curl, verify response matches original file content

### Phase Start
- [ ] T140 [GIT] Verify working tree is clean before starting Phase 6
- [ ] T141 [GIT] Pull and rebase on origin/main if needed
- [ ] T142 [US3] Create specs/001-agentbin-platform/retro/P6.md for this phase
- [ ] T143 [GIT] Commit: chore: initialize phase 6 retro

### Implementation
- [ ] T144 [US3] Implement raw content routes (GET /{uid}/raw — latest, GET /{uid}/v{n}/raw — specific version, serve as Content-Type: text/plain per FR-008/FR-009) in crates/server/src/routes/raw.rs (use devs:rust-dev agent)
- [ ] T145 [US3] Register raw routes in crates/server/src/routes/mod.rs (use devs:rust-dev agent)
- [ ] T146 [GIT] Commit: feat(server): add raw content access routes

### Phase End
- [ ] T147 [US3] Run /sdd:map incremental for Phase 6 changes
- [ ] T148 [GIT] Commit: docs: update codebase documents for phase 6
- [ ] T149 [US3] Review retro/P6.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T150 [GIT] Commit: docs: finalize phase 6 retro
- [ ] T151 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T152 [GIT] Create/update PR to main with phase summary
- [ ] T153 [GIT] Verify all CI checks pass
- [ ] T154 [GIT] Report PR ready status

**Checkpoint**: Raw access works — /{uid}/raw and /{uid}/v{n}/raw serve original content as text/plain

---

## Phase 7: US9 — Upload Management (Priority: P2)

**Goal**: Users list their uploads and delete specific versions via CLI with human and JSON output

**Independent Test**: Upload several files, `agentbin list` shows all, `agentbin delete <uid> --version 1` removes it, list updates, URL returns 404

### Phase Start
- [ ] T155 [GIT] Verify working tree is clean before starting Phase 7
- [ ] T156 [GIT] Pull and rebase on origin/main if needed
- [ ] T157 [US9] Create specs/001-agentbin-platform/retro/P7.md for this phase
- [ ] T158 [GIT] Commit: chore: initialize phase 7 retro

### Implementation
- [ ] T159 [US9] Implement manage routes (GET /api/uploads — list user's uploads with versions per api-spec.md, DELETE /api/uploads/{uid}/v{n} — delete version, require owner or admin) in crates/server/src/routes/manage.rs (use devs:rust-dev agent)
- [ ] T160 [US9] Register manage routes in crates/server/src/routes/mod.rs (use devs:rust-dev agent)
- [ ] T161 [GIT] Commit: feat(server): add upload management routes
- [ ] T162 [P] [US9] Implement list command (GET /api/uploads, display as table with UIDs/versions/URLs or JSON per FR-035/FR-036) in crates/cli/src/commands/list.rs (use devs:rust-dev agent)
- [ ] T163 [P] [US9] Implement delete command (DELETE /api/uploads/{uid}/v{n}, confirm deletion, display result per FR-035) in crates/cli/src/commands/delete.rs (use devs:rust-dev agent)
- [ ] T164 [US9] Wire list and delete commands into CLI dispatch in crates/cli/src/commands/mod.rs (use devs:rust-dev agent)
- [ ] T165 [GIT] Commit: feat(cli): add list and delete commands

### Phase End
- [ ] T166 [US9] Run /sdd:map incremental for Phase 7 changes
- [ ] T167 [GIT] Commit: docs: update codebase documents for phase 7
- [ ] T168 [US9] Review retro/P7.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T169 [GIT] Commit: docs: finalize phase 7 retro
- [ ] T170 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T171 [GIT] Create/update PR to main with phase summary
- [ ] T172 [GIT] Verify all CI checks pass
- [ ] T173 [GIT] Report PR ready status

**Checkpoint**: Upload management works — list shows all uploads with versions, delete removes specific versions

---

## Phase 8: US5 — Admin User Management (Priority: P2)

**Goal**: Admin users add, update, and remove authorized users via CLI. Changes effective immediately without server restart.

**Independent Test**: Admin adds user → new user uploads successfully. Admin removes user → user's uploads rejected. Last-admin removal blocked.

### Phase Start
- [ ] T174 [GIT] Verify working tree is clean before starting Phase 8
- [ ] T175 [GIT] Pull and rebase on origin/main if needed
- [ ] T176 [US5] Create specs/001-agentbin-platform/retro/P8.md for this phase
- [ ] T177 [GIT] Commit: chore: initialize phase 8 retro

### Implementation
- [ ] T178 [US5] Implement admin routes (POST /api/admin/users — add user, PUT /api/admin/users/{username} — update, DELETE /api/admin/users/{username} — remove; admin-only auth guard, last-admin guard per FR-025/FR-026, error codes per api-spec.md) in crates/server/src/routes/admin.rs (use devs:rust-dev agent)
- [ ] T179 [US5] Register admin routes in crates/server/src/routes/mod.rs (use devs:rust-dev agent)
- [ ] T180 [GIT] Commit: feat(server): add admin user management routes
- [ ] T181 [US5] Implement admin CLI command (subcommands: add-user --username --public-key --display-name --is-admin, update-user, remove-user with human/JSON output) in crates/cli/src/commands/admin.rs (use devs:rust-dev agent)
- [ ] T182 [US5] Wire admin command into CLI dispatch in crates/cli/src/commands/mod.rs (use devs:rust-dev agent)
- [ ] T183 [GIT] Commit: feat(cli): add admin user management command

### Phase End
- [ ] T184 [US5] Run /sdd:map incremental for Phase 8 changes
- [ ] T185 [GIT] Commit: docs: update codebase documents for phase 8
- [ ] T186 [US5] Review retro/P8.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T187 [GIT] Commit: docs: finalize phase 8 retro
- [ ] T188 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T189 [GIT] Create/update PR to main with phase summary
- [ ] T190 [GIT] Verify all CI checks pass
- [ ] T191 [GIT] Report PR ready status

**Checkpoint**: Admin management functional — add/update/remove users immediately effective, last-admin guard active

---

## Phase 9: US7 — Upload Metadata & Info Badge (Priority: P2)

**Goal**: Agents attach metadata at upload time (title, description, tags, agent fields, arbitrary key/value). Info badge shows well-known fields with icons, custom fields as "Key: value"

**Independent Test**: Upload with --title "Plan" --tags arch --agent-model claude --meta sprint=42, verify badge shows all fields correctly

### Phase Start
- [ ] T192 [GIT] Verify working tree is clean before starting Phase 9
- [ ] T193 [GIT] Pull and rebase on origin/main if needed
- [ ] T194 [US7] Create specs/001-agentbin-platform/retro/P9.md for this phase
- [ ] T195 [GIT] Commit: chore: initialize phase 9 retro

### Implementation
- [ ] T196 [US7] Add metadata flags to CLI upload command (--title, --description, --tags repeatable, --agent-model, --agent-provider, --agent-tool, --trigger, --meta key=value repeatable per FR-028) in crates/cli/src/commands/upload.rs (use devs:rust-dev agent)
- [ ] T197 [GIT] Commit: feat(cli): add metadata flags to upload command
- [ ] T198 [US7] Enhance badge WebComponent with well-known field icons (title heading, description subtitle, tag chips, agent icon group, trigger icon) and custom field rendering per FR-029/FR-030 in static/badge.js
- [ ] T199 [GIT] Commit: feat: enhance info badge with metadata icons and custom field display
- [ ] T200 [US7] Enhance server badge data construction to serialize full per-version metadata into data-meta attribute in crates/server/src/badge.rs (use devs:rust-dev agent)
- [ ] T201 [GIT] Commit: feat(server): serialize full metadata into badge data attribute

### Phase End
- [ ] T202 [US7] Run /sdd:map incremental for Phase 9 changes
- [ ] T203 [GIT] Commit: docs: update codebase documents for phase 9
- [ ] T204 [US7] Review retro/P9.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T205 [GIT] Commit: docs: finalize phase 9 retro
- [ ] T206 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T207 [GIT] Create/update PR to main with phase summary
- [ ] T208 [GIT] Verify all CI checks pass
- [ ] T209 [GIT] Report PR ready status

**Checkpoint**: Metadata fully functional — all fields flow from CLI → upload → storage → badge display

---

## Phase 10: US6 — Collections (Priority: P3)

**Goal**: Files grouped into named collections with overview pages. Latest version of each file shown. Timeline scrubber reveals historical states.

**Independent Test**: Upload 3 files to collection "review", verify /c/review shows all 3, timeline scrubber shows states at different points

### Phase Start
- [ ] T210 [GIT] Verify working tree is clean before starting Phase 10
- [ ] T211 [GIT] Pull and rebase on origin/main if needed
- [ ] T212 [US6] Create specs/001-agentbin-platform/retro/P10.md for this phase
- [ ] T213 [GIT] Commit: chore: initialize phase 10 retro

### Implementation
- [ ] T214 [US6] Create collection.html askama template (file list with latest version links, client-side timeline scrubber JS with markers per upload timestamp per FR-018/FR-019/FR-020) in crates/server/src/templates/collection.html (use devs:rust-dev agent)
- [ ] T215 [GIT] Commit: feat(server): add collection template with timeline scrubber
- [ ] T216 [US6] Implement collection routes (GET /c/{name} — overview page, POST /api/collections/{name}/members — add file, DELETE /api/collections/{name}/members/{uid} — remove file, auto-delete empty collections per FR-021/api-spec.md) in crates/server/src/routes/collection.rs (use devs:rust-dev agent)
- [ ] T217 [US6] Register collection routes in crates/server/src/routes/mod.rs (use devs:rust-dev agent)
- [ ] T218 [GIT] Commit: feat(server): add collection routes with auto-delete
- [ ] T219 [US6] Add --collection flag to CLI upload command for assigning at upload time per FR-017 in crates/cli/src/commands/upload.rs (use devs:rust-dev agent)
- [ ] T220 [US6] Add collection management subcommands (add-to-collection, remove-from-collection per FR-040) to CLI in crates/cli/src/commands/mod.rs (use devs:rust-dev agent)
- [ ] T221 [GIT] Commit: feat(cli): add collection support

### Phase End
- [ ] T222 [US6] Run /sdd:map incremental for Phase 10 changes
- [ ] T223 [GIT] Commit: docs: update codebase documents for phase 10
- [ ] T224 [US6] Review retro/P10.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T225 [GIT] Commit: docs: finalize phase 10 retro
- [ ] T226 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T227 [GIT] Create/update PR to main with phase summary
- [ ] T228 [GIT] Verify all CI checks pass
- [ ] T229 [GIT] Report PR ready status

**Checkpoint**: Collections functional — files grouped, overview page renders, timeline scrubber works, auto-delete on empty

---

## Phase 11: US8 — File Expiration (Priority: P3)

**Goal**: Per-version TTL in days. Background sweeper removes expired files. Files without expiry persist indefinitely.

**Independent Test**: Upload with --expiry 1, verify file accessible immediately, verify removed after TTL elapses

### Phase Start
- [ ] T230 [GIT] Verify working tree is clean before starting Phase 11
- [ ] T231 [GIT] Pull and rebase on origin/main if needed
- [ ] T232 [US8] Create specs/001-agentbin-platform/retro/P11.md for this phase
- [ ] T233 [GIT] Commit: chore: initialize phase 11 retro

### Implementation
- [ ] T234 [US8] Implement background expiration sweeper (periodic scan for expired versions via expires_at in meta.json, remove files, update upload.json, clean affected collections, log sweeps per FR-031/FR-033/FR-034) in crates/server/src/expiry.rs (use devs:rust-dev agent)
- [ ] T235 [US8] Wire sweeper into server startup as tokio::spawn background task in crates/server/src/main.rs (use devs:rust-dev agent)
- [ ] T236 [GIT] Commit: feat(server): add background file expiration sweeper
- [ ] T237 [US8] Add --expiry flag (days, per-version TTL per FR-031) to CLI upload command in crates/cli/src/commands/upload.rs (use devs:rust-dev agent)
- [ ] T238 [GIT] Commit: feat(cli): add --expiry flag to upload command

### Phase End
- [ ] T239 [US8] Run /sdd:map incremental for Phase 11 changes
- [ ] T240 [GIT] Commit: docs: update codebase documents for phase 11
- [ ] T241 [US8] Review retro/P11.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T242 [GIT] Commit: docs: finalize phase 11 retro
- [ ] T243 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T244 [GIT] Create/update PR to main with phase summary
- [ ] T245 [GIT] Verify all CI checks pass
- [ ] T246 [GIT] Report PR ready status

**Checkpoint**: Expiration works — expired files removed automatically, non-expiring files persist indefinitely

---

## Phase 12: Polish & Deployment

**Purpose**: Dockerfile, fly.toml, structured logging, exit code validation, final quality pass

### Phase Start
- [ ] T247 [GIT] Verify working tree is clean before starting Phase 12
- [ ] T248 [GIT] Pull and rebase on origin/main if needed
- [ ] T249 Create specs/001-agentbin-platform/retro/P12.md for this phase
- [ ] T250 [GIT] Commit: chore: initialize phase 12 retro

### Deployment
- [ ] T251 [P] Create Dockerfile (multi-stage: rust:latest builder with cargo build --release -p agentbin-server, debian:bookworm-slim runtime, EXPOSE 8080) in Dockerfile (use devs:rust-dev agent)
- [ ] T252 [P] Create fly.toml (app name, primary_region, [http_service] internal_port 8080, [mounts] for persistent storage at /data) in fly.toml
- [ ] T253 [GIT] Commit: chore: add Dockerfile and fly.toml for deployment

### Cross-Cutting Concerns
- [ ] T254 Verify structured logging configuration (JSON when AGENTBIN_LOG_FORMAT=json, pretty otherwise; log uploads/deletions/auth failures/expiry sweeps/startup/shutdown per FR-050/FR-051) across crates/server/ (use devs:rust-dev agent)
- [ ] T255 [GIT] Commit: feat(server): verify and enhance structured logging
- [ ] T256 Validate CLI exit codes match spec (0=success, 1=general, 2=auth, 3=not-found, 4=validation, 5=connection per api-spec.md) in crates/cli/ (use devs:rust-dev agent)
- [ ] T257 [GIT] Commit: fix(cli): standardize exit codes per spec

### Final Validation
- [ ] T258 Run full validation: `cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo fmt --all -- --check` (use devs:rust-dev agent)
- [ ] T259 Run `dist plan` and verify expected artifacts for CLI and server binaries
- [ ] T260 [GIT] Commit: fix: resolve any final build/lint/dist issues
- [ ] T261 Run quickstart.md validation — verify all development and testing commands work (use devs:rust-dev agent)
- [ ] T262 [GIT] Commit: fix: resolve any quickstart validation issues

### Phase End
- [ ] T263 Run final codebase mapping (/sdd:map incremental)
- [ ] T264 [GIT] Commit: docs: update codebase documents for final phase
- [ ] T265 Review retro/P12.md and extract critical learnings to CLAUDE.md (conservative)
- [ ] T266 [GIT] Commit: docs: finalize final phase retro
- [ ] T267 [GIT] Push branch to origin (ensure pre-push hooks pass)
- [ ] T268 [GIT] Create/update PR to main with phase summary
- [ ] T269 [GIT] Verify all CI checks pass
- [ ] T270 [GIT] Report PR ready status

**Checkpoint**: All features complete. Lint/test clean. Deployment ready. Release pipeline configured.

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup)
  └→ Phase 2 (Foundational — core lib + server/CLI skeletons)
       └→ Phase 3 (US4: Auth) ←── prerequisite for all uploads
            └→ Phase 4 (US1: Upload & Render) ←── MVP
                 ├→ Phase 5 (US2: Versioning)
                 ├→ Phase 6 (US3: Raw Access)
                 ├→ Phase 7 (US9: Upload Mgmt)
                 ├→ Phase 8 (US5: Admin Mgmt)
                 ├→ Phase 9 (US7: Metadata & Badge)
                 ├→ Phase 10 (US6: Collections)
                 └→ Phase 11 (US8: Expiration)
                      └→ Phase 12 (Polish & Deploy)
```

### User Story Dependencies

| Story | Phase | Depends On | Can Parallel With |
|-------|-------|-----------|-------------------|
| US4 (Auth, P1) | 3 | Foundational | — |
| US1 (Upload & Render, P1) | 4 | US4 | — |
| US2 (Versioning, P2) | 5 | US1 | US3, US5, US7, US9 |
| US3 (Raw Access, P2) | 6 | US1 | US2, US5, US7, US9 |
| US9 (Upload Mgmt, P2) | 7 | US1 | US2, US3, US5, US7 |
| US5 (Admin Mgmt, P2) | 8 | US1, US4 | US2, US3, US7, US9 |
| US7 (Metadata & Badge, P2) | 9 | US1 | US2, US3, US5, US9 |
| US6 (Collections, P3) | 10 | US1 | US8 |
| US8 (Expiration, P3) | 11 | US1 | US6 |

### Parallel Opportunities

**Within Phase 2 (Foundational)**:
```
T031 (error.rs) ║ T032 (uid.rs) ║ T033 (filetype.rs)  — parallel core types
T046 (config.rs) ║ T047 (state.rs)                      — parallel server setup
T050 (request_id.rs) ║ T051 (auth.rs) ║ T052 (mod.rs)   — parallel middleware
T059 (commands/mod.rs) ║ T060 (config.rs) ║ T061 (output.rs) ║ T062 (signing.rs) — parallel CLI
```

**Within Phase 4 (US1 Upload & Render)**:
```
T097 (rendered.html) ║ T098 (plain.html)  — parallel templates
```

**Within Phase 7 (US9 Upload Management)**:
```
T162 (list.rs) ║ T163 (delete.rs)  — parallel CLI commands
```

**Cross-Phase (with multiple developers after MVP)**:
```
Developer A: Phase 5 (Versioning) → Phase 10 (Collections)
Developer B: Phase 6 (Raw) → Phase 7 (Upload Mgmt) → Phase 11 (Expiration)
Developer C: Phase 8 (Admin) → Phase 9 (Metadata & Badge)
```

---

## Implementation Strategy

### MVP First (Phases 1–4)

1. Phase 1: Scaffold workspace + CI/CD configs
2. Phase 2: Build core library + server/CLI skeletons
3. Phase 3: Auth system (US4)
4. Phase 4: Upload & render (US1)
5. **STOP AND VALIDATE**: Upload file → get URL → rendered in browser with badge
6. Deploy to fly.io if ready

### Incremental Delivery

| After Phase | What Works | Value Delivered |
|-------------|-----------|----------------|
| 4 (MVP) | Upload + render + auth + badge | Core product |
| + 5 | Versioning + version banners | Document evolution |
| + 6 | Raw content access | Programmatic consumption |
| + 7 | List + delete uploads | Content management |
| + 8 | Admin user management | Multi-user deployment |
| + 9 | Full metadata in badge | Self-documenting artifacts |
| + 10 | Collections + timeline | Organized narratives |
| + 11 | Auto-expiration | Storage management |
| 12 | Deployment + polish | Production-ready |

---

## Summary

| Metric | Value |
|--------|-------|
| **Total tasks** | 270 |
| **Phases** | 12 |
| **User stories covered** | 9 (US1–US9) |
| **Suggested MVP scope** | Phases 1–4 (US4 + US1) |

### Tasks per User Story

| Story | Phase | Description |
|-------|-------|-------------|
| US4 (Auth, P1) | 3 | Keygen command, verify auth middleware |
| US1 (Upload & Render, P1) | 4 | Upload route, rendering pipeline, view route, badge, CLI upload |
| US2 (Versioning, P2) | 5 | Versioned uploads, versioned URLs, version banner |
| US3 (Raw Access, P2) | 6 | Raw content routes |
| US9 (Upload Mgmt, P2) | 7 | Manage routes, list + delete CLI commands |
| US5 (Admin Mgmt, P2) | 8 | Admin routes, admin CLI command |
| US7 (Metadata & Badge, P2) | 9 | Metadata flags, badge enhancement |
| US6 (Collections, P3) | 10 | Collection routes, template, CLI flags |
| US8 (Expiration, P3) | 11 | Expiry sweeper, --expiry flag |

### Format Validation

All tasks follow required checklist format: `- [ ] [TaskID] [P?] [Story?] Description with file path`
- Sequential IDs (T001–T270)
- [P] on parallelizable tasks
- [US#] on user story phase tasks
- [GIT] on git workflow tasks
- `(use devs:rust-dev agent)` on Rust implementation tasks
- Exact file paths on all implementation tasks

---

## Notes

- All `.rs` tasks reference `devs:rust-dev` agent
- Tests written alongside implementation per constitution II (not as separate TDD tasks)
- Each phase ends with push → PR update → CI check → report PR ready status
- Retro files capture learnings; only critical universal patterns promoted to CLAUDE.md
- Codebase mapping after each phase keeps .sdd/ documents current
- Phase 2 (Foundational) builds ALL core modules upfront for cleaner dependency graph
