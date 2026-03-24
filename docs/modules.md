# Module System

Extensible system for building Dockerfiles from composable, dependency-aware template modules.

## How It Works

Each module is a TOML metadata file paired with a Jinja2 (`.dockerfile.j2`) template. At compile time, all built-in modules are embedded via `include_str!`. At runtime, the registry loads them, the resolver sorts them topologically using `petgraph`, and the renderer feeds them through `minijinja`.

## Module Definition Format

```toml
[module]
name = "example"
category = "lang"          # base | lang | tool | agent | security | custom
description = "..."
version = "1.0.0"

[module.parameters]
version = { type = "string", default = "20", description = "Node.js version" }

[module.dependencies]
requires = ["build-essential"]   # auto-added if missing
conflicts = ["other-module"]     # prevents co-installation
after = ["base"]                 # ordering constraint
```

## Categories

- **base**: OS images (ubuntu, debian, alpine) — mutually exclusive
- **lang**: Programming languages (node, python, rust, go, java, ruby, dotnet, zig, cpp)
- **tool**: Dev tools (git, build_essential, docker_cli)
- **agent**: AI agents (claude_code, codex_cli)
- **security**: Security config (user_setup, firewall)

## Implementation Files

- `src/module/definition.rs` — `ModuleDefinition` struct (deserialized from TOML)
- `src/module/registry.rs` — `ModuleRegistry` loads built-ins and optional user modules
- `src/module/resolver.rs` — `ModuleResolver` resolves dependencies via topological sort
- `src/module/renderer.rs` — `DockerfileGenerator` renders final Dockerfile via minijinja
- `src/module/builtin/` — All built-in module TOML + J2 template pairs
