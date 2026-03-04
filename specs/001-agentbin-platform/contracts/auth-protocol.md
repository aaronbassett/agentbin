# Authentication Protocol: Ed25519 Request Signing

## Overview

Every authenticated API request must include Ed25519 signature headers. The server verifies the signature against registered public keys and rejects invalid, missing, or replayed requests.

## Request Headers

| Header | Required | Description |
|--------|----------|-------------|
| `X-AgentBin-PublicKey` | Yes | Base64-encoded Ed25519 public key |
| `X-AgentBin-Signature` | Yes | Base64-encoded Ed25519 signature |
| `X-AgentBin-Timestamp` | Yes | Unix timestamp (seconds) of request creation |
| `X-Request-Id` | Optional | Client-provided request identifier (echoed in response) |

## Signing Process (CLI Side)

### 1. Construct the signing payload

Concatenate these fields with newline (`\n`) separator:

```
{METHOD}\n
{PATH}\n
{TIMESTAMP}\n
{BODY_HASH}
```

Where:
- `METHOD`: Uppercase HTTP method (GET, POST, PUT, DELETE)
- `PATH`: Full request path including query string (e.g., `/api/upload`)
- `TIMESTAMP`: Unix timestamp in seconds (same value as `X-AgentBin-Timestamp` header)
- `BODY_HASH`: SHA-256 hex digest of the request body (empty string hash for GET/DELETE with no body)

### 2. Sign the payload

```
signature = ed25519_sign(private_key, signing_payload)
```

### 3. Encode and attach headers

```
X-AgentBin-PublicKey: base64(public_key)
X-AgentBin-Signature: base64(signature)
X-AgentBin-Timestamp: {timestamp}
```

## Verification Process (Server Side)

### 1. Extract headers

Parse all three required headers. Reject with `AUTH_MISSING_HEADERS` if any are absent.

### 2. Validate timestamp (replay protection)

```
if abs(server_time - request_timestamp) > 300 seconds (5 minutes):
    reject with AUTH_REPLAY_DETECTED
```

### 3. Look up public key

```
user = users.find_by_public_key(request_public_key)
if user is None:
    reject with AUTH_UNKNOWN_KEY
```

### 4. Reconstruct signing payload

Server reconstructs the same payload the client signed:

```
{METHOD}\n{PATH}\n{TIMESTAMP}\n{BODY_HASH}
```

### 5. Verify signature

```
if not ed25519_verify(public_key, signing_payload, signature):
    reject with AUTH_INVALID_SIGNATURE
```

### 6. Attach user context

On success, attach the authenticated user to the request context for downstream route handlers.

## Key Generation

CLI command: `agentbin keygen`

1. Generate Ed25519 key pair using `ed25519-dalek` with `rand::rngs::OsRng`
2. Save private key to `~/.config/agentbin/key.pem` (base64-encoded, file permissions 0600)
3. Print public key to stdout for sharing with admin

**Key file format** (PEM-like, base64-encoded raw key bytes):

```
-----BEGIN AGENTBIN PRIVATE KEY-----
MC4CAQAwBQYDK2VwBCIEIE3t...
-----END AGENTBIN PRIVATE KEY-----
```

## Security Considerations

- **Replay window**: 5 minutes (300 seconds). Accounts for clock skew between client and server.
- **Body hash**: Prevents body tampering after signing.
- **No nonce storage**: The timestamp-based approach avoids server-side nonce storage. Acceptable for v1 single-instance deployment.
- **Key rotation**: Not supported in v1. Users can generate new keys and re-register.
- **Transport**: HTTPS required in production (TLS termination at fly.io edge).
