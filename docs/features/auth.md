# Auth

Maps authentication configuration to environment variables and volume mounts for Claude and Codex agent containers.

## Scope

**In scope:**
- Claude authentication (6 methods): API key, OAuth, Bedrock IAM, Bedrock API key, Vertex, Proxy
- Codex authentication (4 methods): API key, OAuth, Azure, Custom
- Environment variable generation for each method
- Volume mount generation for credential files (OAuth, Vertex)
- `.env.example` line generation for documentation

**Not in scope:**
- Credential management or storage
- Token refresh or rotation
- Authentication against the actual APIs
- Auth method selection UI (handled by wizard)

## Data/Control Flow

```
AuthConfig (from ProjectConfig)
    │
    ├── claude: Option<ClaudeAuthConfig>
    │     └── method: ClaudeAuthMethod enum
    │
    └── codex: Option<CodexAuthConfig>
          ├── method: CodexAuthMethod enum
          ├── azure_endpoint: Option<String>
          ├── custom_env_key: Option<String>
          └── custom_base_url: Option<String>
              │
              ▼
    auth::claude::requirements(auth, container_user) → AuthRequirements
    auth::codex::requirements(auth, container_user) → AuthRequirements
              │
              ▼
    AuthRequirements {
        env_vars: IndexMap<String, String>,    → docker-compose environment
        volumes: Vec<AuthVolume>,              → docker-compose volumes
        env_example_lines: Vec<String>,        → .env.example content
    }
```

## Types

| Type | Description |
|------|-------------|
| `AuthRequirements` | Output: env vars, volumes, and .env.example lines |
| `AuthVolume` | Volume mount: source, target, read_only flag |
| `ClaudeAuthConfig` | Contains `method: ClaudeAuthMethod` |
| `CodexAuthConfig` | Contains `method`, `azure_endpoint`, `custom_env_key`, `custom_base_url` |

## Claude Authentication Methods

### API Key
- **Env var**: `ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}`
- **Volumes**: None

### OAuth
- **Env vars**: None
- **Volume**: `${HOME}/.claude/.credentials.json` → `/home/{user}/.claude/.credentials.json` (read-only)

### Bedrock (AWS IAM)
- **Env vars**: `CLAUDE_CODE_USE_BEDROCK=1`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_SESSION_TOKEN` (optional, defaults empty), `AWS_REGION`
- **Volumes**: None

### Bedrock API Key (AWS Bearer)
- **Env vars**: `CLAUDE_CODE_USE_BEDROCK=1`, `AWS_BEARER_TOKEN_BEDROCK`, `AWS_REGION`
- **Volumes**: None

### Vertex (Google AI)
- **Env vars**: `CLAUDE_CODE_USE_VERTEX=1`, `GOOGLE_APPLICATION_CREDENTIALS=/home/{user}/.config/gcloud/application_default_credentials.json`, `ANTHROPIC_VERTEX_PROJECT_ID`
- **Volume**: `${GOOGLE_APPLICATION_CREDENTIALS}` → target path (read-only)

### Proxy/Gateway
- **Env vars**: `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`
- **Volumes**: None

## Codex Authentication Methods

### API Key
- **Env var**: `OPENAI_API_KEY=${OPENAI_API_KEY}`
- **Volumes**: None

### OAuth
- **Env vars**: None
- **Volume**: `${HOME}/.codex/auth.json` → `/home/{user}/.codex/auth.json` (read-only)

### Azure
- **Env var**: `AZURE_OPENAI_API_KEY`
- **Optional**: `AZURE_OPENAI_ENDPOINT` (if `azure_endpoint` configured)
- **Volumes**: None

### Custom
- **Env var**: Custom key (from `custom_env_key`, default: `CUSTOM_API_KEY`)
- **Optional**: `OPENAI_BASE_URL` (if `custom_base_url` configured)
- **Volumes**: None

## Public API

| Function | Signature | Description |
|----------|-----------|-------------|
| `claude::requirements()` | `(auth: &ClaudeAuthConfig, user: &str) -> AuthRequirements` | Claude auth env vars + volumes |
| `codex::requirements()` | `(auth: &CodexAuthConfig, user: &str) -> AuthRequirements` | Codex auth env vars + volumes |

## Implementation Files

| File | Role | Key Exports |
|------|------|-------------|
| `src/auth/mod.rs` | Shared types | `AuthRequirements`, `AuthVolume` |
| `src/auth/claude.rs` | Claude auth mapping (6 methods) | `requirements()` |
| `src/auth/codex.rs` | Codex auth mapping (4 methods) | `requirements()` |

## Invariants and Constraints

1. **OAuth methods use volumes, not env vars**: Credential files are mounted read-only into the container.
2. **API key methods use env vars, not volumes**: Keys are passed via environment variables referencing `.env` file.
3. **Container user affects mount paths**: The `container_user` parameter determines the home directory path inside the container (e.g., `/home/dev/`).
4. **Claude and Codex env vars never overlap**: Each agent uses distinct environment variable names.
5. **env_example_lines always present**: Every auth method produces at least one line for the `.env.example` file.
6. **Optional fields produce conditional env vars**: `azure_endpoint`, `custom_env_key`, and `custom_base_url` only add env vars when configured.
7. **AWS_SESSION_TOKEN defaults empty**: For Bedrock IAM, session token is optional and defaults to empty string if not set.
