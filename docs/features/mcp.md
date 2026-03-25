# MCP (Model Context Protocol)

Generates MCP server configurations in two formats: `.mcp.json` for Claude Code discovery and docker-compose sidecar services.

## Scope

**In scope:**
- `.mcp.json` generation (docker run command format for Claude Code)
- Docker Compose sidecar service generation for MCP servers
- Environment variable passthrough
- Volume mount configuration
- Optional command override and port exposure

**Not in scope:**
- MCP protocol implementation
- MCP server discovery or health monitoring
- MCP server image building
- Communication between agent and MCP servers (handled by Docker networking)

## Data/Control Flow

```
[mcp.server-name] in cc-container.toml
    │
    ├── McpServerConfig
    │     ├── image: String
    │     ├── command: Option<Vec<String>>
    │     ├── env: Vec<String>
    │     ├── volumes: Vec<String>
    │     └── port: Option<u16>
    │
    ├──▶ mcp::config::generate_mcp_json(config)
    │     └── For each MCP server:
    │           Build docker run args:
    │             docker run -i --rm --network host
    │               -e KEY1=VAL1
    │               -e KEY2=VAL2
    │               -v /host1:/container1
    │               image:tag
    │               [command args...]
    │           │
    │           ▼
    │         .mcp.json:
    │         {
    │           "mcpServers": {
    │             "server-name": {
    │               "command": "docker",
    │               "args": ["run", "-i", "--rm", ...]
    │             }
    │           }
    │         }
    │
    └──▶ compose::generator (MCP service section)
          └── For each MCP server:
                Create mcp-{name} compose service:
                  ├── image
                  ├── environment (${VAR} substitution)
                  ├── volumes
                  ├── command (optional override)
                  ├── ports (if port specified)
                  └── restart: "unless-stopped"
```

## Config

```toml
[mcp.github]
image = "mcp/github:latest"
command = ["node", "server.js"]
env = ["GITHUB_TOKEN=${GITHUB_TOKEN}"]
volumes = ["/data:/data"]
port = 8000

[mcp.fetch]
image = "mcp/fetch:latest"
env = []
volumes = []
```

## Docker Args Generation Order (for .mcp.json)

The args array is built in this order:
1. `run`, `-i`, `--rm`, `--network`, `host`
2. `-e KEY1=VAL1` (one `-e` flag per env var)
3. `-v /host:/container` (one `-v` flag per volume)
4. `image:tag`
5. Command args (optional, after image)

## Compose Service Generation

Each MCP server becomes a compose service named `mcp-{config-key}` with:
- Image from config
- Environment vars from `env` array (using `${VAR}` substitution syntax)
- Volume mounts from `volumes` array
- Command override (if specified)
- Port mapping as `port:port` (if port specified)
- `restart: "unless-stopped"`
- No healthcheck (not enforced)

Named volumes in MCP configs (e.g., `mcp-data:/data`) are auto-registered in compose top-level `volumes:`. Host-path volumes (starting with `/`) are not.

## Public API

| Function | Signature | Description |
|----------|-----------|-------------|
| `config::generate_mcp_json()` | `(config: &ProjectConfig) -> Result<String>` | Generate `.mcp.json` content |

## Implementation Files

| File | Role | Key Exports |
|------|------|-------------|
| `src/mcp/mod.rs` | Module exports | — |
| `src/mcp/config.rs` | `.mcp.json` generation | `generate_mcp_json()` |
| `src/mcp/service.rs` | Compose service generation (tests) | — |

## Invariants and Constraints

1. **Port field ignored in .mcp.json**: The `port` config option is only used in compose service generation, not in the docker run args for `.mcp.json`.
2. **All MCP servers use `--network host`** in `.mcp.json`: This enables direct host network access for the docker run command.
3. **Compose services use default network**: MCP sidecar services in compose use the default compose bridge network, not host networking.
4. **No healthchecks on MCP services**: Unlike infrastructure services, MCP sidecars don't have healthcheck definitions in compose.
5. **Env var format consistency**: In compose, env vars use `${KEY}` substitution syntax referencing the `.env` file. In `.mcp.json`, env vars are passed as-is.
6. **Named volumes auto-registered**: Volumes that look like named volumes (not path-based) are added to compose top-level `volumes:`.
7. **Empty MCP section produces empty output**: If no `[mcp.*]` sections exist, `generate_mcp_json()` returns `{"mcpServers":{}}`.
