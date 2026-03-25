# CLI & Wizard

## Commands

```
cc-container init           # Interactive project setup wizard
cc-container generate       # Generate Dockerfile, docker-compose.yml, .env.example
cc-container module list    # List available modules
cc-container module info    # Show module details
cc-container module add     # Add modules to config
cc-container service list   # List available infrastructure services
cc-container service info   # Show service details
cc-container mcp            # Manage MCP server configs
cc-container config         # View/edit configuration
cc-container doctor         # Diagnose environment issues
cc-container completions    # Generate shell completions
```

## Implementation Files

- `src/cli/mod.rs` — Clap command tree definition (`Cli`, `Commands` enum, `GlobalArgs`)
- `src/cli/init.rs` — `init` command
- `src/cli/generate.rs` — `generate` command
- `src/cli/module.rs` — `module` subcommand
- `src/cli/service.rs` — `service` subcommand
- `src/cli/mcp.rs` — `mcp` subcommand
- `src/cli/config_cmd.rs` — `config` subcommand
- `src/cli/doctor.rs` — `doctor` command
- `src/wizard/flow.rs` — Interactive wizard main flow
- `src/wizard/prompts.rs` — User prompt helpers
