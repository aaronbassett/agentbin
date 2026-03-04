# Data Model: Agentbin Platform (v1)

## Entities

### 1. Upload

A file stored on the server, identified by a server-generated UID. Has one or more versions.

**Storage**: `{STORAGE}/uploads/{uid}/upload.json`

```json
{
  "uid": "Xn4f8BqR2m",
  "owner": "alice",
  "collection": "feature-review",
  "latest_version": 2,
  "created_at": "2026-03-04T12:00:00Z"
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| uid | String (10 chars, alphanumeric) | Yes | Server-generated via nanoid |
| owner | String | Yes | Username of original uploader |
| collection | String \| null | No | Collection name if assigned |
| latest_version | u32 | Yes | Highest version number |
| created_at | DateTime (ISO 8601) | Yes | First upload timestamp |

### 2. Version

A specific revision of an upload. Has its own URL, content, and metadata.

**Storage**:
- Content: `{STORAGE}/uploads/{uid}/v{n}/content{.ext}`
- Metadata: `{STORAGE}/uploads/{uid}/v{n}/meta.json`

```json
{
  "version": 1,
  "filename": "architecture.html",
  "content_type": "text/html",
  "size_bytes": 12345,
  "uploaded_at": "2026-03-04T12:00:00Z",
  "uploaded_by": "alice",
  "expires_at": "2026-04-03T12:00:00Z",
  "metadata": {
    "title": "Architecture Diagram",
    "description": "System overview for v2 migration",
    "tags": ["architecture", "diagram"],
    "agent": {
      "model": "claude-opus-4-6",
      "provider": "anthropic",
      "tool": "visual-explainer"
    },
    "trigger": "pr-comment",
    "custom": {
      "sprint": "42",
      "reviewer": "alice"
    }
  }
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| version | u32 | Yes | Starts at 1, increments per upload |
| filename | String | Yes | Original filename |
| content_type | String | Yes | MIME type (detected from extension) |
| size_bytes | u64 | Yes | File size |
| uploaded_at | DateTime | Yes | This version's upload timestamp |
| uploaded_by | String | Yes | Username of uploader |
| expires_at | DateTime \| null | No | Computed from TTL at upload time |
| metadata | Metadata | No | Well-known + arbitrary fields |

### 3. Metadata

Per-version metadata. Well-known fields have defined types; arbitrary key/value pairs stored in `custom`.

| Well-Known Field | Type | Info Badge Display |
|-----------------|------|-------------------|
| title | String | Heading text |
| description | String | Subtitle text |
| tags | String[] | Tag chips |
| agent.model | String | Icon + model name |
| agent.provider | String | Icon + provider name |
| agent.tool | String | Icon + tool name |
| trigger | String | Icon + trigger context |

Arbitrary metadata: `custom` map of `String → String`, displayed as "**Key:** value" in the info badge.

### 4. User

An authorized uploader. Stored in the server's user configuration.

**Storage**: `{STORAGE}/users.json`

```json
{
  "users": {
    "alice": {
      "public_key": "MCowBQYDK2VwAyEA...",
      "display_name": "Alice",
      "is_admin": true,
      "created_at": "2026-03-04T12:00:00Z"
    },
    "bob": {
      "public_key": "MCowBQYDK2VwAyEA...",
      "display_name": "Bob",
      "is_admin": false,
      "created_at": "2026-03-05T09:00:00Z"
    }
  }
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| username (key) | String | Yes | Unique identifier |
| public_key | String (base64) | Yes | Ed25519 public key |
| display_name | String | No | Human-readable name |
| is_admin | bool | Yes | Admin flag for user management |
| created_at | DateTime | Yes | Registration timestamp |

**Constraints**:
- At least one user must have `is_admin: true` at all times (FR-025)
- Removed users retain their uploads but can no longer authenticate (spec clarification)
- Atomic writes: read-modify-write with temp file + rename (FR-026)

### 5. Collection

A named group of uploads with an overview page.

**Storage**: `{STORAGE}/collections/{name}.json`

```json
{
  "name": "feature-review",
  "members": [
    { "uid": "Xn4f8BqR2m", "added_at": "2026-03-04T12:00:00Z" },
    { "uid": "Yp5g9CrS3n", "added_at": "2026-03-04T14:00:00Z" }
  ],
  "created_at": "2026-03-04T12:00:00Z"
}
```

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| name | String | Yes | URL-safe collection identifier |
| members | Array of {uid, added_at} | Yes | Ordered list of member uploads |
| created_at | DateTime | Yes | Collection creation timestamp |

**Constraints**:
- Auto-deleted when all members removed (FR-021)
- Files can be added/removed via CLI commands (FR-040)
- Collection pages show latest version of each member by default (FR-018)

### 6. Key Pair (CLI-side only)

Ed25519 key pair stored on the user's machine.

**Storage**: `~/.config/agentbin/key.pem` (or configured path)

| Component | Format | Location |
|-----------|--------|----------|
| Private key | Base64-encoded Ed25519 secret key | Local file (CLI) |
| Public key | Base64-encoded Ed25519 public key | Registered on server (users.json) |

## File Type Detection

Based on file extension only (FR-049). No content sniffing.

| Extension(s) | Content Type | Rendering |
|-------------|--------------|-----------|
| `.html`, `.htm` | text/html | Served as-is with badge injection |
| `.md`, `.markdown` | text/markdown | Rendered to HTML via comrak |
| `.json` | application/json | Syntax highlighted |
| `.jsonc` | application/json | Syntax highlighted |
| `.toml` | application/toml | Syntax highlighted |
| `.yaml`, `.yml` | text/yaml | Syntax highlighted |
| `.xml` | application/xml | Syntax highlighted |
| `.rst` | text/x-rst | Syntax highlighted (no full render, FR-004) |
| `.txt` and all others | text/plain | Plain text in minimal HTML wrapper |

## URL Patterns

| URL | Purpose | Response |
|-----|---------|----------|
| `/{uid}` | Latest rendered version | Rendered HTML |
| `/{uid}/v{n}` | Specific rendered version | Rendered HTML |
| `/{uid}/raw` | Latest raw content | text/plain |
| `/{uid}/v{n}/raw` | Specific raw content | text/plain |
| `/c/{name}` | Collection overview | Collection HTML page |
| `/api/upload` | Create new upload | JSON (uid, url, version) |
| `/api/upload/{uid}` | Upload new version | JSON (uid, url, version) |
| `/api/uploads` | List user's uploads | JSON array |
| `/api/uploads/{uid}/v{n}` | Delete version | JSON (status) |
| `/api/admin/users` | Add user | JSON (status) |
| `/api/admin/users/{username}` | Update/remove user | JSON (status) |
| `/api/collections/{name}/members` | Add to collection | JSON (status) |
| `/api/collections/{name}/members/{uid}` | Remove from collection | JSON (status) |
| `/health` | Health check | 200 OK |
| `/_static/badge.js` | Info badge WebComponent | application/javascript |

## Storage Layout

```text
{AGENTBIN_STORAGE_PATH}/
├── uploads/
│   ├── Xn4f8BqR2m/
│   │   ├── upload.json
│   │   ├── v1/
│   │   │   ├── content.html
│   │   │   └── meta.json
│   │   └── v2/
│   │       ├── content.html
│   │       └── meta.json
│   └── Yp5g9CrS3n/
│       ├── upload.json
│       └── v1/
│           ├── content.md
│           └── meta.json
├── collections/
│   └── feature-review.json
├── users.json
└── .tmp/                              # Atomic write staging area
```

## State Transitions

### Upload Lifecycle

```
New Upload → upload.json created, v1/ directory created
  → New Version → v{n}/ directory created, upload.json.latest_version updated
  → Delete Version → v{n}/ directory removed
    → If last version → upload.json removed, UID directory removed
  → Expire → Same as Delete Version (triggered by sweeper)
```

### Collection Lifecycle

```
First file added → {name}.json created
  → Files added/removed → members array updated
  → Last file removed → {name}.json deleted
```

### User Lifecycle

```
Admin registers user → users.json updated (atomic)
  → User uploads files → normal upload lifecycle
  → Admin removes user → users.json updated (atomic)
    → User's uploads remain accessible (ownership recorded)
    → User can no longer authenticate
```
