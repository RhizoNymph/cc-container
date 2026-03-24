# MCP (Model Context Protocol)

Generates MCP server sidecar service configurations for docker-compose.

## Config

```toml
[mcp.server-name]
image = "mcp/fetch:latest"
command = ["node", "server.js"]
env = ["API_KEY=${MCP_API_KEY}"]
volumes = ["/data:/data"]
port = 8000
```

## Implementation Files

- `src/mcp/config.rs` — MCP config structures
- `src/mcp/service.rs` — MCP service generation for compose
