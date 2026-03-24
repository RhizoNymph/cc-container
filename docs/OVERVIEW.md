# cc-container Overview

CLI tool that generates containerized AI coding agent environments (Dockerfiles, docker-compose stacks, firewall rules) from a TOML config file.

## Core Pipeline

```
cc-container.toml → Config loading/merge/validation → Module resolution → Dockerfile + Compose generation
```

## Features

- [Module System](./modules.md) — Extensible template-based Dockerfile generation with dependency resolution
- [Compose Generation](./compose.md) — Typed docker-compose.yml output with 16 built-in service templates
- [Firewall](./firewall.md) — iptables rule generation from domain/CIDR allowlists
- [Auth](./auth.md) — Environment variable mapping for Claude and Codex authentication methods
- [MCP](./mcp.md) — Model Context Protocol server sidecar configuration
- [Config](./config.md) — Project and user config loading, merging, and validation
- [CLI & Wizard](./cli.md) — Command structure and interactive init flow

## Source Layout

```
src/
├── main.rs          # Entry point
├── error.rs         # Error types
├── cli/             # Command implementations
├── config/          # Config loading/merge/validation
├── module/          # Module system (registry, resolver, renderer, builtins)
├── compose/         # Docker Compose generation + service templates
├── firewall/        # Firewall rule generation
├── auth/            # Auth env var mapping
├── mcp/             # MCP server config generation
└── wizard/          # Interactive setup flow
```
