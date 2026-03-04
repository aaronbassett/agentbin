# Feature Specification: Agentbin Platform (v1)

**Feature Branch**: `001-agentbin-platform`
**Created**: 2026-03-04
**Status**: Draft
**Input**: User description: "A file sharing service that lets AI agents publish rendered documents at public URLs"
**Codebase Documentation**: See [.sdd/codebase/](.sdd/codebase/) for technical details

## Clarifications

### Session 2026-03-04

- Q: Is metadata (title, description, tags, agent fields, arbitrary key/value pairs) stored per-version or per-upload (UID level)? → A: Per-version — each version has its own independent metadata.
- Q: Which cryptographic algorithm should be used for key pairs and request signing? → A: Ed25519 — fast, compact signatures, simple API, well-supported in Rust via `ed25519-dalek`.
- Q: When a user is removed from the authorized user list, what happens to their existing uploads? → A: Retain — uploads remain accessible at their URLs, ownership recorded as "former user."
- Q: Can there be multiple admin users, and how is admin status granted? → A: Multiple admins — admins can grant/revoke admin flag on other users, with a guard preventing removal of the last admin.
- Q: What validation rules should apply to user-supplied UIDs? → A: Server always generates safe short UIDs on first upload. Users never supply their own UIDs; they receive the generated UID and use it for subsequent version uploads.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Agent Uploads and Shares a Rendered Document (Priority: P1)

An AI agent generates a visual artifact (an HTML architecture diagram, a markdown report, a JSON data summary) and needs to share it with a human. The agent uses the CLI to upload the file, receives a public URL, and posts that URL in a PR comment or Slack message. The human clicks the link and sees the document rendered in its intended visual form — HTML as a live page, markdown as formatted text, JSON with syntax highlighting.

**Why this priority**: This is the core value proposition. Without upload-and-render, nothing else matters. A single successful upload that returns a viewable URL proves the product works.

**Independent Test**: Can be fully tested by uploading a sample HTML file via CLI and verifying the returned URL serves the rendered page in a browser.

**Acceptance Scenarios**:

1. **Given** a user has a valid key pair configured, **When** they upload an HTML file via the CLI, **Then** the CLI returns a public URL and the URL serves the HTML as a rendered page.
2. **Given** a user uploads a Markdown file, **When** a visitor opens the returned URL, **Then** they see the Markdown rendered as formatted HTML with proper headings, lists, and code blocks.
3. **Given** a user uploads a JSON file, **When** a visitor opens the returned URL, **Then** they see the JSON displayed with syntax highlighting in a readable format.
4. **Given** a user uploads a TOML, YAML, or XML file, **When** a visitor opens the returned URL, **Then** the content is displayed with appropriate syntax highlighting.
5. **Given** a user uploads a plain text file, **When** a visitor opens the returned URL, **Then** the text is displayed in a clean, minimal HTML wrapper that preserves the plain-text feel.
6. **Given** a user uploads a reStructuredText file, **When** a visitor opens the returned URL, **Then** the content is displayed with syntax highlighting as plain text (full rST rendering is deferred).
7. **Given** any rendered page, **When** a visitor views it, **Then** a small info badge is visible that, when clicked, shows file metadata (uploader, upload time, version, and any custom metadata).

---

### User Story 2 — Agent Updates a Document with Versioning (Priority: P2)

An agent produces a living document (e.g., an implementation plan) that evolves as work progresses. Each time the agent updates the plan, it uploads a new version using the same UID. Each version has its own URL, and visitors to older versions see a notice that a newer version exists.

**Why this priority**: Versioning transforms agentbin from a one-shot paste service into a tool that tracks the evolution of agent work over time. This is key for workflows where documents are iterated on.

**Independent Test**: Can be tested by uploading a file, then uploading again with the same UID, and verifying both versions have distinct URLs and the old version shows a "newer version available" notice.

**Acceptance Scenarios**:

1. **Given** a file was previously uploaded and the user received UID "abc123", **When** the user uploads a new file specifying that UID, **Then** a new version is created with its own unique URL and the version number increments.
2. **Given** version 1 of a file exists, **When** a visitor views version 1 after version 2 has been uploaded, **Then** a visible notice indicates that a newer version is available with a link to it.
3. **Given** a user uploads a new file (no existing UID), **When** the upload completes, **Then** the server generates a unique UID and returns it alongside the URL.
4. **Given** a versioned file, **When** a visitor accesses the base URL (without version number), **Then** they see the most recent version.

---

### User Story 3 — Accessing Raw Source Content (Priority: P2)

A developer or automated tool needs the original source of an uploaded file — not the rendered version. Every file has a parallel "raw" URL that serves the original content as `text/plain`, usable with `curl` or `wget`.

**Why this priority**: Raw access is essential for programmatic consumption and complements the rendered view. Without it, the platform locks content into browser-only access.

**Independent Test**: Can be tested by uploading a file, then fetching the raw URL with `curl` and verifying the response matches the original file content byte-for-byte.

**Acceptance Scenarios**:

1. **Given** a file has been uploaded, **When** a user requests the raw URL, **Then** the original file content is returned with `Content-Type: text/plain`.
2. **Given** the rendered URL follows a predictable pattern, **When** a user modifies the URL according to the documented pattern, **Then** they get the raw version without needing to look it up separately.

---

### User Story 4 — Key-Based Authentication (Priority: P1)

A user generates a cryptographic key pair using the CLI. They provide their public key to an admin, who registers it on the server. From that point, the user's uploads are authenticated via request signing — no passwords, no sessions, no registration flow.

**Why this priority**: Authentication is a prerequisite for uploads. Without it, the server is an open relay. Key-based auth is the security foundation.

**Independent Test**: Can be tested by generating a key pair, registering the public key, uploading a file with the private key, and verifying that an upload without valid signing is rejected.

**Acceptance Scenarios**:

1. **Given** a user runs the key generation command, **When** the command completes, **Then** a public/private key pair is created and stored locally.
2. **Given** a user's public key is registered on the server, **When** they upload a file with a properly signed request, **Then** the server accepts the upload.
3. **Given** a request with an invalid or missing signature, **When** it reaches the server, **Then** the server rejects it with a clear error message.
4. **Given** a properly signed request, **When** a replay of the same request is sent later, **Then** the server rejects it (replay protection).

---

### User Story 5 — Admin Manages Authorized Users (Priority: P2)

An admin user manages who can upload to agentbin. They add new users by registering public keys, update user details, and remove users who should no longer have access — all via CLI commands, without redeploying the server.

**Why this priority**: User management is needed before any multi-user deployment. Without it, the only way to authorize users is to manually edit server configuration files.

**Independent Test**: Can be tested by having an admin add a new user via CLI, verifying the new user can upload, then removing them and verifying uploads are rejected.

**Acceptance Scenarios**:

1. **Given** a user flagged as admin, **When** they run the add-user command with a username and public key, **Then** the new user is registered and can immediately upload files.
2. **Given** an admin, **When** they run the remove-user command for an existing user, **Then** that user's uploads are rejected going forward.
3. **Given** a non-admin user, **When** they attempt to run user management commands, **Then** the commands are rejected with a permissions error.
4. **Given** an admin adds or removes a user, **When** the change is made, **Then** it takes effect without requiring a server restart.

---

### User Story 6 — Organizing Files into Collections (Priority: P3)

An agent working on a feature produces multiple related artifacts — a visual plan, a diff review, test results. These are grouped into a collection, which has its own overview page showing the latest version of each file. A timeline scrubber lets viewers see the state of the collection at any point in time, revealing how the agent's work evolved.

**Why this priority**: Collections add organizational context that transforms isolated uploads into a coherent narrative. While valuable, individual uploads (P1) and versioning (P2) deliver standalone value first.

**Independent Test**: Can be tested by uploading three files to the same collection, verifying the collection page shows all three, then using the timeline scrubber to view historical states.

**Acceptance Scenarios**:

1. **Given** a user uploads a file with a collection name specified, **When** the upload completes, **Then** the file appears on the collection's overview page.
2. **Given** a collection has multiple files, **When** a visitor views the collection page, **Then** they see the most recent version of each file listed.
3. **Given** a collection with files uploaded at different times, **When** a visitor uses the timeline scrubber and selects a past timestamp, **Then** the page shows file versions as they existed at that moment.
4. **Given** a user removes the last file from a collection, **When** the removal is complete, **Then** the collection is automatically deleted.
5. **Given** a user, **When** they run the collection management command to add or remove a file from a collection, **Then** the collection is updated accordingly.

---

### User Story 7 — Upload Metadata and Info Badge (Priority: P2)

An agent attaches metadata when uploading — a title, description, tags, the model that generated the artifact, the tool or skill that triggered the upload. This metadata is visible via the info badge on every rendered page: well-known fields get icons and special formatting, arbitrary key/value pairs are displayed as labeled text.

**Why this priority**: Metadata provides provenance and context, making artifacts self-documenting. The info badge is the primary interface for understanding what an artifact is and where it came from.

**Independent Test**: Can be tested by uploading a file with various metadata fields and verifying the info badge displays them correctly with appropriate formatting.

**Acceptance Scenarios**:

1. **Given** a user uploads a file with `title`, `description`, and `tags` metadata, **When** a visitor clicks the info badge, **Then** these fields are displayed with recognizable formatting.
2. **Given** a user uploads a file with `agent.model`, `agent.provider`, and `agent.tool` metadata, **When** a visitor views the info badge, **Then** agent-related fields are displayed with appropriate icons or grouping.
3. **Given** a user uploads a file with arbitrary key/value metadata (e.g., `sprint: 42`, `reviewer: alice`), **When** a visitor views the info badge, **Then** each pair is displayed as "**Key:** value".
4. **Given** any rendered page (including uploaded HTML), **When** it loads, **Then** the info badge is injected and isolated from the page's own styles and scripts.

---

### User Story 8 — File Expiration (Priority: P3)

A user uploads a file with an expiration period in days. After the TTL expires, the file is automatically removed from the server and its URL returns a "not found" response. Files without an expiration persist indefinitely.

**Why this priority**: Expiration prevents unbounded storage growth and is important for operational health, but the platform is fully functional without it for initial launches.

**Independent Test**: Can be tested by uploading a file with a 1-day expiry, advancing time or waiting, and verifying the URL returns not found.

**Acceptance Scenarios**:

1. **Given** a user uploads a file with `expiry: 7`, **When** 7 days have elapsed, **Then** the file is no longer accessible and its URL returns a not-found response.
2. **Given** a user uploads a file without specifying an expiry, **When** any amount of time passes, **Then** the file remains accessible indefinitely.
3. **Given** a versioned file where version 1 has a 30-day expiry and version 2 has no expiry, **When** 30 days pass, **Then** version 1 is removed but version 2 remains.

---

### User Story 9 — Upload Management (Priority: P2)

A user lists their own uploads and deletes specific versions of files they no longer need. The CLI provides structured output suitable for both human reading and machine parsing (via `--json` flag).

**Why this priority**: Users need to manage their content. Listing and deleting are basic CRUD operations that make the platform usable beyond fire-and-forget uploads.

**Independent Test**: Can be tested by uploading several files, listing them via CLI, deleting one, and verifying the list updates accordingly.

**Acceptance Scenarios**:

1. **Given** a user has uploaded multiple files, **When** they run the list command, **Then** they see all their uploads with UIDs, versions, upload times, and URLs.
2. **Given** a user wants to remove a specific version, **When** they run the delete command with the UID and version, **Then** that version is removed and its URL returns not found.
3. **Given** any CLI command, **When** the `--json` flag is provided, **Then** the output is structured JSON suitable for programmatic consumption.
4. **Given** a CLI command encounters an error, **When** `--json` is enabled, **Then** the error is returned as a structured JSON object with a clear error message.

---

### Edge Cases

- What happens when a user uploads a file larger than 1MB? The server rejects it with a clear error indicating the size limit.
- What happens when a user uploads a file with an unrecognized file extension? The server falls back to serving it as plain text in a minimal HTML wrapper.
- What happens when a user tries to version a file they don't own? The server rejects the upload with an authorization error.
- What happens when the server receives a malformed signature? The request is rejected with a descriptive error, and no partial state is created.
- What happens when an HTML upload contains malicious scripts? The info badge is isolated from the uploaded content (via WebComponent or iframe), preventing interference. The uploaded HTML is served as-is — agentbin does not sanitize uploads, as it serves agent-generated content, not user-generated content from untrusted sources.
- What happens when two uploads with the same UID arrive simultaneously? The server processes them sequentially, assigning version numbers in arrival order. No data corruption occurs.
- What happens when a file expires while it belongs to a collection? The file is removed from the collection. If it was the last file, the collection is deleted.
- What happens when a user deletes a file that's in a collection? The file is removed from the collection. If it was the last file, the collection is deleted.
- What happens when the server receives a termination signal during an upload? In-flight requests are completed (with a bounded timeout) before shutdown. No partial files are left on disk.
- What happens when the CLI cannot reach the server? The CLI returns a descriptive connection error with a non-zero exit code, both in human and JSON output modes.
- What happens when an admin removes a user? The user can no longer upload or manage files, but all their existing uploads remain accessible at their URLs. Ownership metadata is preserved as "former user."
- What happens when the last admin tries to remove or demote themselves? The server rejects the operation with an error to prevent admin lockout.

## Requirements *(mandatory)*

### Functional Requirements

#### Upload & Rendering

- **FR-001**: System MUST accept file uploads via authenticated CLI requests and return a public URL.
- **FR-002**: System MUST render uploaded HTML files as live web pages (served with `text/html` content type).
- **FR-003**: System MUST render uploaded Markdown files as formatted HTML with proper typography.
- **FR-004**: ~~Deferred~~ — reStructuredText rendering is deferred to a future version. rST files are treated as plain text (served in a minimal HTML wrapper with syntax highlighting).
- **FR-005**: System MUST render uploaded structured data files (JSON, JSONC, TOML, YAML, XML) as syntax-highlighted HTML pages.
- **FR-006**: System MUST wrap uploaded plain text files in a minimal HTML shell that preserves the plain-text appearance.
- **FR-007**: System MUST enforce a 1MB file size limit per upload, rejecting larger files with a clear error.
- **FR-008**: System MUST provide a raw view URL for every file that serves the original content as `text/plain`.
- **FR-009**: The raw view URL MUST follow a predictable, documented pattern derivable from the rendered URL.

#### Info Badge

- **FR-010**: System MUST inject an info badge (WebComponent or iframe) into every rendered page, including uploaded HTML.
- **FR-011**: The info badge MUST be DOM-isolated from the uploaded content to prevent style or script interference.
- **FR-012**: Clicking the info badge MUST reveal a popover displaying: uploader identity, upload timestamp, version number, well-known metadata fields with icons, and arbitrary key/value metadata pairs.

#### Versioning

- **FR-013**: System MUST support versioned uploads — uploading with an existing UID creates a new version.
- **FR-014**: Each version MUST have its own unique URL, with the first upload being version 1.
- **FR-015**: Previous versions MUST display a visible notice/banner linking to the newest version.
- **FR-016**: System MUST generate a unique, short, URL-safe UID for every new upload. UIDs are always server-generated — users do not supply their own. Users receive the UID in the upload response and use it for subsequent version uploads.

#### Collections

- **FR-017**: System MUST support optional grouping of files into named collections at upload time.
- **FR-018**: Collection overview pages MUST display the most recent version of each file by default.
- **FR-019**: Collection pages MUST include a timeline scrubber with markers for each upload timestamp.
- **FR-020**: Selecting a point on the timeline scrubber MUST show file versions as they existed at that moment.
- **FR-021**: Collections MUST be automatically deleted when all files are removed from them.

#### Authentication & User Management

- **FR-022**: System MUST authenticate uploads via Ed25519 public/private key signing with replay protection.
- **FR-023**: The CLI MUST provide a command to generate an Ed25519 key pair.
- **FR-024**: The server MUST maintain a configuration of authorized users and their public keys.
- **FR-025**: Admin users MUST be able to add, update, and remove authorized users (including granting/revoking admin status) via CLI commands without server redeployment. The system MUST prevent removal or demotion of the last remaining admin.
- **FR-026**: Configuration changes from admin commands MUST be persisted atomically (no partial writes).
- **FR-027**: The server MUST reject requests with invalid, missing, or replayed signatures with descriptive error messages.

#### Metadata

- **FR-028**: System MUST accept optional metadata at upload time: well-known fields (`title`, `description`, `tags`, `collection`, `expiry`, `agent.model`, `agent.provider`, `agent.tool`, `trigger`) and arbitrary key/value pairs.
- **FR-029**: Well-known metadata fields MUST receive icons or special formatting in the info badge.
- **FR-030**: Arbitrary metadata key/value pairs MUST be displayed as "**Key:** value" in the info badge.

#### File Expiration

- **FR-031**: System MUST support optional per-version TTL specified in days at upload time.
- **FR-032**: Files without an expiry MUST persist indefinitely.
- **FR-033**: Expired files MUST be removed and their URLs MUST return a not-found response.
- **FR-034**: Expiration MUST be per-version — different versions of the same UID can have different TTLs.

#### Upload Management

- **FR-035**: The CLI MUST provide commands to list a user's uploads and delete specific versions.
- **FR-036**: All CLI commands MUST support a `--json` flag for structured, machine-readable output.
- **FR-037**: Error output in JSON mode MUST include a clear error message in a structured format.

#### Operational

- **FR-038**: System MUST provide a `/health` endpoint returning 200 OK.
- **FR-039**: System MUST store all uploads as files on disk with JSON metadata sidecars (no database).
- **FR-040**: CLI MUST support adding and removing files from collections after initial upload.
- **FR-041**: The server MUST handle termination signals gracefully — completing in-flight requests (with a bounded timeout) and flushing pending writes before exit.
- **FR-042**: All file writes MUST be atomic (write-to-temp-then-rename) to prevent partial or corrupt data from concurrent operations or crashes.
- **FR-043**: The platform MUST run as a single server instance with a single persistent storage volume. Multi-instance deployment is out of scope for v1.

#### CLI-Server Communication

- **FR-044**: The CLI communicates with the server exclusively over HTTP. All CLI operations (upload, list, delete, user management) are API calls to the server.
- **FR-045**: CLI exit codes MUST be predictable and documented: 0 for success, non-zero for errors, with distinct codes for authentication errors, not-found errors, and validation errors.

#### Configuration

- **FR-046**: Server configuration (storage path, listen address, admin credentials) MUST be managed via environment variables.
- **FR-047**: The CLI MUST support configuring the server URL and key file location via environment variables, config file, or CLI flags, with a documented precedence order.

#### Security

- **FR-048**: Rendered content responses MUST include appropriate security headers to restrict what uploaded HTML can execute (content security policy, content type options, frame restrictions).
- **FR-049**: File type detection MUST be based on file extension. Content sniffing is not performed in v1.

#### Logging

- **FR-050**: The server MUST emit structured logs to stdout, with log levels configurable via environment variable.
- **FR-051**: Key events MUST be logged: uploads, deletions, authentication failures, expiration sweeps, startup, and shutdown.

#### Request Tracing

- **FR-052**: Every HTTP response MUST include a unique request identifier header. If the client provides one, the server echoes it; otherwise, the server generates one.

### Key Entities

- **Upload**: A file stored on the server. Identified by a server-generated, short, URL-safe UID. Has one or more versions. Belongs to zero or one collections. Associated with metadata (well-known and arbitrary fields). May have an expiration TTL per version.
- **Version**: A specific revision of an upload. Has its own URL, upload timestamp, file content, and metadata. Identified by UID + version number. Version 1 is the first upload; subsequent uploads with the same UID increment the version. Metadata is stored per-version — each version has its own independent title, description, tags, agent fields, and arbitrary key/value pairs.
- **User**: An authorized uploader. Identified by username and public key. May be flagged as admin. Multiple users can hold the admin flag. Stored in the server's user configuration.
- **Collection**: A named group of uploads. Has an overview page with timeline scrubber. Auto-deletes when empty. Files can be added at upload time or via management commands.
- **Info Badge**: An injected UI element on every rendered page. Displays metadata popover. DOM-isolated from page content.
- **Key Pair**: An Ed25519 public/private key pair used for request signing. Generated by the CLI. Public key registered on the server; private key stored locally by the user.

## Non-Functional Requirements

- **NFR-001**: All CLI output in `--json` mode MUST be predictable and parseable by LLM agents.
- **NFR-002**: The server MUST handle concurrent uploads without data corruption.
- **NFR-003**: The info badge MUST load without degrading the rendered page's performance.
- **NFR-004**: URL patterns for rendered views, raw views, versions, and collections MUST be predictable and documented.

## Development Standards

- **DS-001**: All code MUST pass linting (clippy with pedantic) and formatting checks with zero warnings before merge.
- **DS-002**: Pre-commit hooks MUST run formatting and lint checks automatically before each commit.
- **DS-003**: A CI pipeline MUST run on all pull requests and pushes to main, gating on build, test, lint, and format checks.
- **DS-004**: All commits MUST follow the Conventional Commits format (e.g., `feat:`, `fix:`, `docs:`, `chore:`) to support automated changelog generation.

## Assumptions

- The service targets small-to-medium documents (under 1MB). Large file hosting is not a goal.
- Uploaded HTML is agent-generated and treated as semi-trusted. The platform does not sanitize uploads but isolates the info badge from uploaded content. Security headers (CSP) restrict what uploaded HTML can execute.
- A single admin bootstraps the first user configuration. The initial admin setup is a manual process (e.g., the deployer creates the first admin entry in the config).
- File type detection is based on file extension, not content inspection.
- The timeline scrubber on collection pages is a client-side UI component — the server provides the data (file versions with timestamps), and the frontend renders the timeline interactively.
- Private key storage security is left to the user's operating system and tooling (file permissions, OS keychain, etc.). The CLI stores the key locally but does not enforce a specific security mechanism in v1.
- URL structure and filesystem organization will be designed during implementation planning. UIDs are always server-generated, short, and URL-safe. Time-sortable identifiers are preferred for chronological ordering.
- The platform runs as a single instance with a persistent volume. Multi-instance coordination is out of scope.
- The CLI is an HTTP client to the server — it does not access the filesystem directly. All operations go through the server's API.
- Info badge injection into HTML content happens at serve time (on-the-fly), not at upload time. Non-HTML rendered content (Markdown, JSON viewers) is wrapped in an HTML shell that includes the badge.
- The server's user configuration (authorized public keys) is stored as a file on the persistent volume, read on each request or watched for changes — implementation detail deferred to planning.
- Error responses use a consistent taxonomy with defined categories (auth errors, validation errors, not-found errors, server errors) mapped to appropriate status codes and CLI exit codes.

## Deferred Features

- reStructuredText rendering (full rST-to-HTML conversion)
- Multi-region deployment / CDN
- Collection-level access controls
- Embed mode (iframe widget)
- Search / discovery across uploads
- Bulk upload (directory of files)
- Customizable themes
- API rate limiting
- Configurable file size limits
- Analytics (view counts, referrers)

## Out of Scope

- No comments, annotations, reactions, or feedback on uploads
- No user registration, login, profiles, or sessions
- No database-backed storage
- No hosting of applications, APIs, or interactive apps
- No wikis, in-browser editing, or rich text editors

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An agent can upload a file and receive a shareable URL in a single CLI command, completing the operation in under 5 seconds on a standard connection.
- **SC-002**: All supported file types (HTML, Markdown, JSON, JSONC, TOML, YAML, XML, plain text) render correctly when viewed at their public URL. reStructuredText files display with syntax highlighting as plain text.
- **SC-003**: 100% of upload attempts without valid authentication are rejected.
- **SC-004**: Versioned uploads preserve all previous versions with distinct, stable URLs that remain accessible.
- **SC-005**: The info badge is visible and functional on every rendered page, including uploaded HTML, without interfering with the page content.
- **SC-006**: Admin user management operations (add, update, remove) take effect immediately without server restart.
- **SC-007**: Collection overview pages correctly display the latest version of each file and the timeline scrubber accurately reflects historical states.
- **SC-008**: All CLI commands produce valid, parseable JSON when the `--json` flag is used.
- **SC-009**: The service remains operational and responsive under normal load with the `/health` endpoint returning 200 OK.
- **SC-010**: Expired files are no longer accessible after their TTL elapses.
