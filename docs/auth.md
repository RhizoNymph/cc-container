# Auth

Maps authentication configuration to environment variables for AI agent containers.

## Supported Methods

**Claude:** api-key, oauth, bedrock, bedrock-api-key, vertex, proxy
**Codex:** api-key, oauth, azure, custom

## Implementation Files

- `src/auth/claude.rs` — Claude auth environment variable mapping
- `src/auth/codex.rs` — Codex auth environment variable mapping
