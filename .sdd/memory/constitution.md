<!--
Sync Impact Report
====================
Version change: N/A → 1.0.0
Added principles:
  - I. Workspace Architecture (NON-NEGOTIABLE)
  - II. Pragmatic Testing
  - III. Safe by Default
  - IV. Ship Simply
  - V. Deployable Always
Added sections:
  - Development Standards (Rust-specific)
  - Deployment Policy (fly.io)
  - Governance
Removed sections: None
Templates requiring updates:
  - plan-template.md: ✅ Compatible (Constitution Check section present)
  - spec-template.md: ✅ Compatible (no constitution-specific changes needed)
  - tasks-template.md: ✅ Compatible (phase structure supports workspace crate tasks)
Follow-up TODOs: None
-->

# agentbin Constitution

## Core Principles

### I. Workspace Architecture (NON-NEGOTIABLE)

agentbin is a Cargo workspace. All functionality lives in
purpose-driven crates with clear boundaries.

**Rules**:
- A shared core library crate contains domain logic
- CLI and web server are separate binary crates
- Each crate MUST compile and test independently
- Cross-crate dependencies flow inward (binaries depend
  on core, never the reverse)

**Why**: Enforces separation of concerns, enables independent
testing, and allows the CLI and server to evolve at different
paces without coupling.

**Enforcement**: `cargo build -p <crate>` and
`cargo test -p <crate>` MUST succeed for every crate in
isolation.

### II. Pragmatic Testing

Test critical paths. Skip trivial code.

**Rules**:
- Business logic in core crate MUST have unit tests
- API endpoints MUST have integration tests covering
  happy path and primary error cases
- CLI commands MUST have tests for argument parsing and
  expected output
- Trivial getters, simple struct construction, and
  boilerplate code do NOT require dedicated tests
- Bug fixes MUST include a regression test

**Why**: Solo developer with balanced priorities. Tests
protect against regressions where it matters without
creating maintenance burden on low-risk code.

**Enforcement**: `cargo test --workspace` MUST pass before
any merge to main. CI blocks on test failure.

### III. Safe by Default

Rust's safety guarantees are a feature. Extend them with
strict tooling.

**Rules**:
- `#![deny(unsafe_code)]` in all crates unless explicitly
  justified in a comment block
- `clippy::pedantic` enabled; warnings treated as errors
  in CI
- No `.unwrap()` or `.expect()` in production code paths;
  use proper error handling with `thiserror` or `anyhow`
- Input validation at system boundaries (HTTP requests,
  CLI arguments, file I/O)
- Secrets MUST NOT appear in code, logs, or error messages

**Why**: Prevents entire classes of bugs at compile time.
Strict linting catches issues before they reach production.

**Enforcement**: CI runs `cargo clippy -- -D warnings` and
`cargo fmt -- --check`. Pre-commit hooks recommended.

### IV. Ship Simply

Start with the simplest solution. Add complexity only when
proven necessary.

**Rules**:
- No persistence layer until a feature explicitly requires
  state (start stateless or file-based)
- Minimize external dependencies; prefer std library where
  reasonable
- No premature abstractions: three similar lines of code
  are better than a premature trait
- Features MUST be scoped to current requirements, not
  hypothetical future needs (YAGNI)
- One way to do things; avoid offering multiple
  configuration paths for the same behavior

**Why**: Solo developer building an MVP. Every abstraction
and dependency is ongoing maintenance cost. Complexity is
the enemy of shipping.

**Enforcement**: PR/review checks for unnecessary
abstractions. Constitution Check in implementation plans
MUST flag added complexity.

### V. Deployable Always

The main branch MUST always be in a deployable state.

**Rules**:
- All changes to main pass CI (build, test, clippy, fmt)
- Health check endpoint required for the web server
- Structured logging (JSON) for production; human-readable
  for development
- fly.io deployment MUST be reproducible from a single
  command (`fly deploy`)
- Environment-specific configuration via environment
  variables, never hardcoded

**Why**: A broken main branch blocks all progress. Reliable
deployment reduces friction between writing code and
shipping it.

**Enforcement**: CI pipeline gates merges to main. Deployment
smoke test after each deploy.

## Development Standards

### Rust Toolchain
- **Edition**: 2021 (or latest stable)
- **MSRV**: Latest stable Rust at project start
- **Formatter**: `rustfmt` with default configuration
- **Linter**: `clippy` with `pedantic` lint group
- **Error handling**: `thiserror` for library errors,
  `anyhow` for application/binary errors
- **Async runtime**: `tokio` (required for web framework)

### Web Framework
- Framework selection is deferred until the first feature
  requiring HTTP endpoints. Evaluate Axum, Actix Web, and
  Rocket at that time based on current ecosystem fit.
- Decision MUST be recorded as an amendment to this
  constitution.

### Repository Structure
```
agentbin/
├── Cargo.toml          # Workspace root
├── crates/
│   ├── core/           # Shared domain logic
│   ├── cli/            # CLI binary
│   └── server/         # Web server binary
├── .sdd/               # SDD artifacts
└── fly.toml            # fly.io configuration
```

## Deployment Policy

- **Platform**: fly.io
- **Build**: Multi-stage Dockerfile (builder + runtime)
- **Config**: Environment variables via `fly secrets`
- **Health**: `/health` endpoint returning 200 OK
- **Regions**: Single region for MVP; expand when needed

## Governance

This constitution is the highest authority for development
decisions in agentbin. All implementation plans, code
reviews, and architectural choices MUST comply with these
principles.

### Amendments

1. Propose change with rationale in a commit message or PR
2. Document what principle is changing and why
3. Update version number per semantic versioning
4. Update Last Amended date
5. If principle removal or redefinition: MAJOR version bump

### Exceptions

Exceptions to any principle (except Principle I) are
permitted when:
1. The exception is documented with rationale
2. Simpler alternatives were attempted or explained
3. The exception is time-bounded where possible

Exceptions are tracked in the Complexity Tracking table
of the relevant implementation plan.

### Compliance Review

Every implementation plan MUST include a Constitution Check
section verifying compliance with all five principles before
work begins.

**Version**: 1.0.0 | **Ratified**: 2026-03-04 | **Last Amended**: 2026-03-04
