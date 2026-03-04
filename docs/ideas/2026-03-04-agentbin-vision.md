# agentbin Vision

## The Idea

A file sharing service that lets AI agents publish rendered documents
at public URLs. Agents upload files via a CLI and receive a shareable
link they can drop into PR comments, Slack messages, or anywhere else.
Unlike pastebin or gist services, agentbin actually *renders* the
content — HTML is served as a live page, Markdown becomes formatted
HTML, structured data gets syntax highlighting.

## Problem Space

AI coding agents produce rich visual artifacts — HTML reports,
architecture diagrams, visual plans, diff reviews — but have no
native way to share them. Current options (gists, pastebins, wikis)
serve content as plain text or source code, stripping away the visual
presentation that makes these artifacts valuable. Agents need a
programmatic way to publish rendered documents and get a URL back
that they can reference in their workflows.

**Who has this problem:**
- Developers working with AI agents that produce visual reports
- AI agents themselves, which need to "show their work" to humans
- Teams where agent-generated plans and reviews need to be shared
  across PR comments, chat, and other collaboration surfaces

## Core Value Proposition

Give AI agents a voice beyond the terminal — a way to show humans
rich, rendered documents at stable, shareable URLs.

## Key Features (v1 Scope)

### Server

- **Authenticated uploads** — public/private key signing with tamper
  protection. Server maintains a config file of authorized users and
  their public keys. No registration, no sessions, no passwords.
- **Admin user management** — users flagged as admin can add, update,
  and remove other users via CLI commands without redeploying the
  server. Config changes persisted via atomic write.
- **Smart rendering by file type:**
  - HTML: served as-is with injected info badge
  - Markdown, reStructuredText: rendered to HTML
  - JSON, JSONC, TOML, YAML, XML: syntax-highlighted HTML page
  - Plain text: wrapped in minimal HTML shell, kept looking plain
- **Injected info badge** — a small icon/button (isolated as
  WebComponent or iframe) injected into every rendered page,
  including uploaded HTML. Clicking it reveals a popover with file
  metadata: uploader, upload time, version, well-known fields with
  icons/special rendering, plus arbitrary key/value pairs displayed
  as `**Key:** value`.
- **Versioning** — uploading with an existing UID creates a new
  version. Each version has its own unique URL (same base, different
  version number; first upload is version 1). Previous versions
  display a notice/banner that a newer version exists.
- **Collections** — optional grouping specified at upload time.
  Collection overview page shows the most recent version of each
  file by default. Includes a timeline scrubber with markers for
  each upload timestamp; clicking a point shows file versions as
  they existed at that moment. Collections are auto-deleted when
  all files are removed from them.
- **Raw view** — every file has a parallel URL that serves the
  original source as text/plain. Predictable URL pattern so it
  works with wget/curl.
- **File expiration** — optional per-version TTL set in days at
  upload time. No expiry by default.
- **File size limit** — 1MB hardcoded for v1.
- **Filesystem storage** — all uploads stored as files on disk,
  organized by directories, with JSON metadata sidecars. No
  database.
- **Health check endpoint** — `/health` returning 200 OK.

### CLI

- **Upload file** — submit a file and receive a public URL. Accept
  optional UID for versioned uploads; auto-generate UID for new
  files.
- **Upload metadata** — optional fields at upload time:
  - Well-known: `title`, `description`, `tags`, `collection`, `expiry` (days), `agent.model`,
    `agent.provider`, `agent.tool`, `trigger`
  - Arbitrary: any additional key/value pairs
- **Collection management** — add/remove a file from a collection.
  (Adding can also be done at upload time.)
- **Upload management** — list uploads, delete specific versions.
- **User management (admin only)** — add, update, remove authorized
  users on the server.
- **Key pair generation** — generate public/private key pair for
  authentication.
- **Machine-friendly output** — all commands support `--json` flag for structured output designed for LLM agents. Predictable output. Clear error messages.

### Shared Core Crate

- Crypto: key generation, request signing, signature verification,
  replay protection
- File type detection and metadata structures
- UID and version handling logic

## Deferred Features

- Multi-region deployment / CDN for served files
- Collection-level access controls (private collections)
- Embed mode (iframe-able widget for embedding in other pages)
- Search/discovery across all public uploads
- Bulk upload (directory of files in one command)
- Customizable themes for rendered views
- API rate limiting
- Configurable file size limits
- Analytics (view counts, referrer tracking)

## Out of Scope / Anti-Goals

- **Not a collaboration tool** — no comments, annotations,
  reactions, or feedback on uploads. Discussion happens where the
  link is posted (PR comments, Slack, etc). Splitting conversation
  across multiple places is an anti-goal.
- **Not a user management system** — no registration, login,
  profiles, or sessions. Auth is key-based, users are entries in
  a config file managed by admins.
- **Not a database-backed application** — filesystem storage only.
  Accept the scaling ceiling this implies. A database would violate
  the project's Ship Simply principle.
- **Not a general-purpose hosting platform** — serves rendered
  documents, not applications, APIs, or interactive apps.
- **Not a replacement for documentation systems** — no wikis, no
  in-browser editing, no rich text editors.
- **Not a pastebin clone** — the value is rendering and presenting
  content visually, not just storing text. Every other sharing
  service serves HTML as text/plain. Agentbin serves it rendered.

## Open Questions

- **WebComponent vs iframe for the info badge** — both provide DOM
  isolation from uploaded HTML content. WebComponent is lighter
  weight; iframe is stronger isolation. Decision deferred to
  implementation.
- **Private key storage on local device** — the CLI needs the
  private key for signing. How to keep it secure? OS keychain
  integration, file permissions, or environment variable? Needs
  investigation during implementation.
- **Tamper protection mechanism** — signed payload is the likely approach,
  but exact scheme TBD.
- **URL structure** — the exact URL pattern for files, versions,
  raw views, and collections needs design during implementation.
- **Filesystem organization** — directory structure for stored
  files and metadata sidecars needs design. Needs to support
  efficient lookup by UID and by collection.

## Inspirations & Analogies

- **Filmmaking dailies** — a director reviews raw footage at the
  end of each shooting day to stay aligned with the crew. Agent
  uploads are dailies; collections are the review room where a
  human sees the evolution of an agent's thinking over time.
- **Museum curation** — each artifact stands alone, but the
  arrangement tells a story. Collections present multiple agent
  artifacts as a coherent narrative of a feature's development.
- **Printmaking editions** — numbered prints from the same plate.
  Each version is "the same work" but distinct, and each has its
  own identity (URL).
- **Natural history museum placards** — specimens always have a
  context card explaining what you're looking at. The info badge
  serves this role: provenance, creation context, version history.
- **Library cataloging** — structured fields (author, title, ISBN)
  plus freeform keywords. Well-known metadata fields get special
  rendering; arbitrary key/values provide an escape valve.
- **Fine art provenance** — knowing who created an artifact and
  in what context (which model, which skill, which PR) is part of
  the artifact's value.
