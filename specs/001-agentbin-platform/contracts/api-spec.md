# API Specification: Agentbin Platform (v1)

All API endpoints are under `/api/`. Authenticated endpoints require Ed25519 signature headers (see [auth-protocol.md](auth-protocol.md)).

## Upload Endpoints

### POST /api/upload — Create New Upload

**Auth**: Required

**Request**: `multipart/form-data`

| Part | Type | Required | Notes |
|------|------|----------|-------|
| file | binary | Yes | File content (max 1MB) |
| metadata | JSON string | No | Metadata object (see below) |

**Metadata JSON** (optional fields):
```json
{
  "title": "Architecture Diagram",
  "description": "System overview",
  "tags": ["architecture"],
  "collection": "feature-review",
  "expiry": 30,
  "agent": {
    "model": "claude-opus-4-6",
    "provider": "anthropic",
    "tool": "visual-explainer"
  },
  "trigger": "pr-comment",
  "custom": {
    "sprint": "42"
  }
}
```

**Response** (201 Created):
```json
{
  "uid": "Xn4f8BqR2m",
  "version": 1,
  "url": "https://agentbin.dev/Xn4f8BqR2m",
  "raw_url": "https://agentbin.dev/Xn4f8BqR2m/raw"
}
```

**Errors**:
- 401: Invalid/missing signature
- 413: File exceeds 1MB limit
- 422: Invalid metadata format

---

### POST /api/upload/{uid} — Upload New Version

**Auth**: Required (must be owner of the UID)

**Request**: Same as POST /api/upload

**Response** (201 Created):
```json
{
  "uid": "Xn4f8BqR2m",
  "version": 2,
  "url": "https://agentbin.dev/Xn4f8BqR2m/v2",
  "raw_url": "https://agentbin.dev/Xn4f8BqR2m/v2/raw"
}
```

**Errors**:
- 401: Invalid/missing signature
- 403: Not the owner of this UID
- 404: UID not found
- 413: File exceeds 1MB limit

---

## Management Endpoints

### GET /api/uploads — List User's Uploads

**Auth**: Required

**Response** (200 OK):
```json
{
  "uploads": [
    {
      "uid": "Xn4f8BqR2m",
      "latest_version": 2,
      "collection": "feature-review",
      "created_at": "2026-03-04T12:00:00Z",
      "versions": [
        {
          "version": 1,
          "filename": "plan.html",
          "size_bytes": 12345,
          "uploaded_at": "2026-03-04T12:00:00Z",
          "url": "https://agentbin.dev/Xn4f8BqR2m/v1",
          "expires_at": null
        },
        {
          "version": 2,
          "filename": "plan.html",
          "size_bytes": 14567,
          "uploaded_at": "2026-03-04T14:00:00Z",
          "url": "https://agentbin.dev/Xn4f8BqR2m/v2",
          "expires_at": null
        }
      ]
    }
  ]
}
```

---

### DELETE /api/uploads/{uid}/v{version} — Delete Version

**Auth**: Required (must be owner or admin)

**Response** (200 OK):
```json
{
  "deleted": true,
  "uid": "Xn4f8BqR2m",
  "version": 1
}
```

**Errors**:
- 401: Invalid/missing signature
- 403: Not the owner and not admin
- 404: UID or version not found

---

## Admin Endpoints

### POST /api/admin/users — Add User

**Auth**: Required (admin only)

**Request** (JSON):
```json
{
  "username": "bob",
  "public_key": "MCowBQYDK2VwAyEA...",
  "display_name": "Bob",
  "is_admin": false
}
```

**Response** (201 Created):
```json
{
  "username": "bob",
  "created_at": "2026-03-05T09:00:00Z"
}
```

**Errors**:
- 401: Invalid/missing signature
- 403: Not an admin
- 409: Username already exists

---

### PUT /api/admin/users/{username} — Update User

**Auth**: Required (admin only)

**Request** (JSON):
```json
{
  "display_name": "Robert",
  "is_admin": true
}
```

**Response** (200 OK):
```json
{
  "username": "bob",
  "updated": true
}
```

**Errors**:
- 401: Invalid/missing signature
- 403: Not an admin
- 404: User not found

---

### DELETE /api/admin/users/{username} — Remove User

**Auth**: Required (admin only)

**Response** (200 OK):
```json
{
  "username": "bob",
  "removed": true
}
```

**Errors**:
- 401: Invalid/missing signature
- 403: Not an admin, or trying to remove last admin
- 404: User not found

---

## Collection Endpoints

### POST /api/collections/{name}/members — Add File to Collection

**Auth**: Required (must own the file)

**Request** (JSON):
```json
{
  "uid": "Xn4f8BqR2m"
}
```

**Response** (200 OK):
```json
{
  "collection": "feature-review",
  "uid": "Xn4f8BqR2m",
  "added": true
}
```

---

### DELETE /api/collections/{name}/members/{uid} — Remove File from Collection

**Auth**: Required (must own the file or be admin)

**Response** (200 OK):
```json
{
  "collection": "feature-review",
  "uid": "Xn4f8BqR2m",
  "removed": true,
  "collection_deleted": false
}
```

Note: `collection_deleted: true` if this was the last member (FR-021).

---

## Health Endpoint

### GET /health

**Auth**: None

**Response** (200 OK):
```json
{
  "status": "ok"
}
```

---

## Common Response Headers

Every response includes:

| Header | Description |
|--------|-------------|
| `X-Request-Id` | Echoed from client header, or server-generated UUID (FR-052) |
| `Content-Type` | `application/json` for API responses |

---

## Error Response Format

All errors follow a consistent format:

```json
{
  "error": {
    "code": "AUTH_INVALID_SIGNATURE",
    "message": "Request signature verification failed",
    "status": 401
  }
}
```

**Error codes**:

| Code | Status | Description |
|------|--------|-------------|
| AUTH_MISSING_HEADERS | 401 | Required auth headers not present |
| AUTH_INVALID_SIGNATURE | 401 | Signature verification failed |
| AUTH_REPLAY_DETECTED | 401 | Timestamp outside acceptable window |
| AUTH_UNKNOWN_KEY | 401 | Public key not registered |
| FORBIDDEN | 403 | Insufficient permissions |
| FORBIDDEN_LAST_ADMIN | 403 | Cannot remove/demote last admin |
| NOT_FOUND | 404 | Resource not found |
| CONFLICT | 409 | Resource already exists |
| PAYLOAD_TOO_LARGE | 413 | File exceeds 1MB limit |
| VALIDATION_ERROR | 422 | Invalid request format |
| INTERNAL_ERROR | 500 | Unexpected server error |

**CLI exit codes** (FR-045):

| Exit Code | Meaning |
|-----------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Authentication error |
| 3 | Not found |
| 4 | Validation error |
| 5 | Connection error |
