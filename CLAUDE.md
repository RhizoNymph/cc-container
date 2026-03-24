# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

cc-container is a Rust CLI that generates containerized AI coding agent environments (Claude Code / Codex). It produces Dockerfiles, docker-compose.yml files, .env templates, and firewall rules from a `cc-container.toml` project config.

## Build & Test Commands

```bash
cargo build                    # dev build
cargo build --release          # release build
cargo test                     # run all tests (unit only, no integration test dir)
cargo test <test_name>         # run a single test by name
cargo clippy                   # lint
cargo fmt                      # format
cargo run -- <subcommand>      # run the CLI (e.g. cargo run -- generate)
```

No custom Makefile or task runner — all commands go through Cargo. Snapshot tests use `insta`; update snapshots with `cargo insta review`.

## Architecture

### Pipeline: Config → Modules → Dockerfile + Compose

1. **Config** (`src/config/`): Loads `cc-container.toml` (project config) and `~/.config/cc-container/config.toml` (user config), merges them, and validates.
2. **Module system** (`src/module/`): Each module is a TOML metadata file + Jinja2 template pair under `src/module/builtin/`. The registry loads all built-ins via `include_str!`. The resolver uses `petgraph` for topological sorting of module dependencies. The renderer (`DockerfileGenerator`) feeds resolved modules into `minijinja` to produce the final Dockerfile.
3. **Compose generation** (`src/compose/`): Builds a typed `docker-compose.yml` using the `docker-compose-types` crate. Service templates in `src/compose/service_templates/` define 16 infrastructure services (postgres, redis, kafka, etc.) with health checks, volumes, and env vars.
4. **Firewall** (`src/firewall/`): Generates iptables rules from allowed domains/CIDRs in config.
5. **Auth** (`src/auth/`): Maps auth config to environment variables for Claude and Codex.
6. **MCP** (`src/mcp/`): Generates MCP (Model Context Protocol) server sidecar configs.
7. **Wizard** (`src/wizard/`): Interactive `init` flow using `dialoguer`.

### Module definition pattern

Each built-in module lives in `src/module/builtin/<category>/<name>/` with:
- `<name>.toml` — metadata: name, category, parameters with defaults, dependency declarations (`requires`, `conflicts`, `after`)
- `<name>.dockerfile.j2` — Jinja2 template for Dockerfile instructions

Module categories: `base` (OS images), `lang` (languages), `tool` (dev tools), `agent` (AI agents), `security` (user setup, firewall).

### CLI structure

`src/cli/mod.rs` defines the clap command tree. Each subcommand has its own file in `src/cli/` with a `run()` function. Entry point is `src/main.rs`.

### Error handling

All errors go through `src/error.rs` (`Error` enum with `thiserror`). Functions return `crate::error::Result<T>`. Do not use `.unwrap()` — handle all error cases.

## Key Conventions

- **Embedded assets**: Module TOML + templates are compiled into the binary via `include_str!`. Changes to `.toml` or `.j2` files require recompilation.
- **Ordered maps**: `IndexMap` is used instead of `HashMap` to preserve insertion order in configs and generated output.
- **Typed compose output**: Docker Compose YAML is generated through `docker-compose-types` structs, not raw string templates.
- **Config format**: Project config is TOML (`cc-container.toml`). Generated output includes YAML (compose) and JSON (MCP configs).
