# CLI & Wizard

Clap-based command-line interface with 8 commands and an interactive project initialization wizard.

## Scope

**In scope:**
- CLI command tree (clap derive API)
- Global options (target-dir, config path, verbosity, color)
- 8 commands: init, generate, module, service, mcp, config, doctor, completions
- Interactive wizard flow (dialoguer prompts)
- Shell completion generation

**Not in scope:**
- Actual generation logic (delegated to compose, module, firewall subsystems)
- Docker execution or container management
- Remote operations

## Data/Control Flow

```
main.rs
    │
    ├── Cli::parse() via clap
    │     ├── Global args: --target-dir, --config, -v/-q, --color
    │     └── Commands enum dispatch
    │
    ├── Tracing setup:
    │     ├── -q → error only
    │     ├── (default) → warn
    │     ├── -v → info
    │     ├── -vv → debug
    │     └── -vvv → trace
    │
    └── match command:
          ├── init → init::run()
          │     ├── --no-interactive → generate_default_config()
          │     ├── --template → generate_template_config()
          │     └── (default) → wizard::flow::run()
          │
          ├── generate → generate::run()
          │     ├── load_effective_config()
          │     ├── validate_config() → print warnings
          │     └── Generate targets (--only filter):
          │           ├── dockerfile → DockerfileGenerator::generate()
          │           ├── compose → compose::generator::generate()
          │           ├── firewall → firewall::generator::generate()
          │           ├── env → compose::env::generate_env_example()
          │           ├── mcp → mcp::config::generate_mcp_json()
          │           └── helm → helm::chart::generate() (opt-in only)
          │
          ├── module → module::run()
          │     ├── list → show available modules (optional --category filter)
          │     ├── info → show module details
          │     ├── add → add modules to config (--with key=val)
          │     ├── remove → remove modules from config
          │     └── create → scaffold custom module (--name, --dir)
          │
          ├── service → service::run()
          │     ├── list → show available service templates (optional --category filter)
          │     ├── info → show service template details
          │     ├── add → add services to config (--with key=val)
          │     └── remove → remove services from config
          │
          ├── mcp → mcp::run()
          │     ├── list → show configured MCP servers
          │     ├── add → add MCP server (--image, --command, --env, --volume)
          │     └── remove → remove MCP server
          │
          ├── config → config_cmd::run()
          │     ├── show → display effective config (--format toml|json|yaml)
          │     ├── validate → validate config files
          │     ├── set → set config value by dotted path
          │     ├── get → get config value by dotted path
          │     └── edit → open config in $EDITOR
          │
          ├── doctor → doctor::run()
          │     ├── Check Docker installed (which docker)
          │     ├── Check Docker Compose plugin (docker compose version)
          │     ├── Check config file exists + valid
          │     ├── Validate OAuth credential files (if applicable)
          │     └── Report summary (--verbose for details)
          │
          └── completions → clap_complete::generate()
                └── Generate for: bash, zsh, fish, powershell, elvish
```

## Command Reference

### Global Options

| Option | Description | Default |
|--------|-------------|---------|
| `--target-dir <PATH>` | Project directory | Current working directory |
| `--config <PATH>` | Config file path | `<target-dir>/cc-container.toml` |
| `-v`, `-vv`, `-vvv` | Verbosity (info, debug, trace) | warn |
| `-q`, `--quiet` | Suppress output | false |
| `--color <MODE>` | Color mode: auto, always, never | auto |

### init

```
cc-container init [OPTIONS]
  --template <TEMPLATE>   Starter template: claude, codex, both, minimal
  --agent <AGENT>         Agent type: claude, codex, both
  --no-interactive        Skip prompts, use defaults
```

Validates no existing `cc-container.toml` before proceeding.

### generate

```
cc-container generate [OPTIONS]
  --output <PATH>         Output directory (default: target-dir)
  --dry-run               Print to stdout instead of writing files
  --only <TARGETS>        Comma-separated: dockerfile, compose, firewall, env, mcp, helm
```

### module

```
cc-container module list [--category <CAT>]
cc-container module info <NAME>
cc-container module add <NAMES>... [--with KEY=VAL]
cc-container module remove <NAMES>...
cc-container module create --name <NAME> [--dir <PATH>]
```

### service

```
cc-container service list [--category <CAT>]
cc-container service info <NAME>
cc-container service add <NAMES>... [--with KEY=VAL]
cc-container service remove <NAMES>...
```

### mcp

```
cc-container mcp list
cc-container mcp add <NAME> --image <IMG> [--command <CMD>] [--env KEY=VAL]... [--volume /h:/c]...
cc-container mcp remove <NAME>
```

### config

```
cc-container config show [--format <FMT>]   # toml, json, yaml
cc-container config validate
cc-container config set <KEY> <VALUE>        # dotted path (e.g., image.base)
cc-container config get <KEY>
cc-container config edit                     # opens $EDITOR
```

### doctor

```
cc-container doctor [--verbose]
```

### completions

```
cc-container completions <SHELL>   # bash, zsh, fish, powershell, elvish
```

## Interactive Wizard

The wizard (`wizard::flow::run()`) guides users through project setup with 10 prompts:

| Step | Prompt Type | Options | Default |
|------|-------------|---------|---------|
| 1 | Project name | Text input | Directory name |
| 2 | Agent type | FuzzySelect | Claude Code |
| 3 | Base OS | FuzzySelect | Ubuntu 24.04 |
| 4 | Shell | FuzzySelect | bash |
| 5 | Claude auth | FuzzySelect (if Claude) | API Key |
| 6 | Codex auth | FuzzySelect (if Codex) | API Key |
| 7 | Languages | MultiSelect | Node.js (always included) |
| 8 | Tools | MultiSelect | Git |
| 9 | Services | MultiSelect | None |
| 10 | Firewall | Confirm | No |

**Key behaviors:**
- Node.js is always included (required by agents)
- Auth prompts only appear for selected agent type(s)
- If firewall enabled, auto-adds NET_ADMIN and NET_RAW to runtime capabilities
- Language toolchains have version defaults (node:22, python:3.12, rust:stable, etc.)
- Services are pre-configured with default ports

## Helper Functions

| Function | Description |
|----------|-------------|
| `parse_key_val(s: &str) -> Result<(String, String)>` | Parses `key=value` strings (handles `=` in values) |

## Implementation Files

| File | Role | Key Exports |
|------|------|-------------|
| `src/cli/mod.rs` | Clap command tree | `Cli`, `Commands`, `GlobalArgs` |
| `src/cli/init.rs` | init command | `run()` |
| `src/cli/generate.rs` | generate command | `run()` |
| `src/cli/module.rs` | module subcommands | `run()` |
| `src/cli/service.rs` | service subcommands | `run()` |
| `src/cli/mcp.rs` | mcp subcommands | `run()` |
| `src/cli/config_cmd.rs` | config subcommands | `run()` |
| `src/cli/doctor.rs` | doctor command | `run()` |
| `src/wizard/mod.rs` | Wizard module exports | — |
| `src/wizard/flow.rs` | Interactive init flow | `run()` |
| `src/wizard/prompts.rs` | Prompt helpers | dialoguer wrappers |
| `src/main.rs` | Entry point | Tracing setup, command dispatch |
| `src/error.rs` | Error types | `Error` enum (17 variants) |

## Invariants and Constraints

1. **Config file must not exist for init**: The init command checks for existing `cc-container.toml` and refuses to overwrite.
2. **All commands use `load_effective_config()`**: Commands that need config load both project and user configs and merge them.
3. **Generate respects `--only` filter**: When `--only` is specified, only those targets are generated.
4. **Dry-run is read-only**: `--dry-run` prints to stdout without writing any files.
5. **Verbosity and quiet are mutually exclusive**: `--quiet` overrides any `-v` flags.
6. **Error exit code**: All errors exit with code 1 after printing to stderr.
7. **No interactive flags in subcommands**: Only `init` has `--no-interactive`; all other commands are non-interactive.
8. **Key=value parsing handles embedded `=`**: The `parse_key_val` helper correctly splits on the first `=` only.
