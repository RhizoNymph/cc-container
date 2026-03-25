# Config

Handles loading, merging, and validating project and user configuration.

## Scope

**In scope:**
- Loading project config from `cc-container.toml`
- Loading user config from `~/.config/cc-container/config.toml`
- Merging user defaults into project config (project takes precedence)
- Validating the merged config (port conflicts, auth completeness, firewall requirements)
- All config type definitions (ProjectConfig and all nested types)

**Not in scope:**
- Config serialization for output (handled by CLI commands)
- Interactive config creation (handled by wizard)
- Config file discovery beyond the two known paths

## Data/Control Flow

```
load_effective_config(path)
    │
    ├── load_project_config(path)
    │     └── Read file → toml::from_str → ProjectConfig
    │         Also parse raw toml::Value for merge detection
    │
    ├── load_user_config()
    │     └── Read ~/.config/cc-container/config.toml → Option<UserConfig>
    │         Returns None if file doesn't exist
    │
    ├── merge_configs(&mut project, &user, raw_image)
    │     ├── Check raw TOML to detect explicit vs default values
    │     ├── Merge image settings (base, base_version, shell, platform)
    │     ├── Merge modules (skip auto-managed: ubuntu, debian, alpine,
    │     │     claude-code, codex-cli, firewall)
    │     ├── Deep merge module params (user params are fallback)
    │     └── Propagate user-setup params to image.user and image.shell
    │
    └── validate_config(&config) → Result<Vec<ValidationWarning>>
          ├── Check auth configured for selected agent type
          ├── Check port conflicts (primary + secondary ports across all services)
          ├── Validate port ranges (1–65535)
          ├── Check firewall enabled → NET_ADMIN in cap_add
          └── Return warnings (non-fatal) or errors (fatal)
```

## Config Types

### Root Types

| Type | Description |
|------|-------------|
| `ProjectConfig` | Root struct for `cc-container.toml`. Contains all sections below. |
| `UserConfig` | User-level defaults from `~/.config/cc-container/config.toml`. |
| `UserDefaults` | Optional defaults: base, base_version, shell, platform, modules. |

### Nested Config Sections

| Type | Fields | Defaults |
|------|--------|----------|
| `ProjectMeta` | `name` (required), `description` | — |
| `AgentConfig` | `agent_type` (required), `claude_version`, `codex_version` | versions: "latest" |
| `ImageConfig` | `base`, `base_version`, `platform`, `tag`, `user`, `shell` | Ubuntu, linux/amd64, "dev", Bash |
| `AuthConfig` | `claude` (Option), `codex` (Option) | None |
| `FirewallConfig` | `enabled`, `allowed_domains`, `allowed_cidrs`, `allow_ssh`, `allow_dns` | false, [], [], true, true |
| `WorkspaceConfig` | `mount_path`, `additional_mounts` | "/workspace", [] |
| `EnvironmentConfig` | `env_files`, `vars` | None, {} |
| `RuntimeConfig` | `cap_add`, `cap_drop`, `security_opt`, `memory_limit`, `cpu_limit`, `shm_size` | [], [], [], None, None, None |

### Enums

| Enum | Variants |
|------|----------|
| `AgentType` | `Claude`, `Codex`, `Both` |
| `BaseOs` | `Ubuntu`, `Debian`, `Alpine` |
| `ShellType` | `Bash`, `Zsh`, `Sh` |
| `ClaudeAuthMethod` | `ApiKey`, `Oauth`, `Bedrock`, `BedrockApiKey`, `Vertex`, `Proxy` |
| `CodexAuthMethod` | `ApiKey`, `Oauth`, `Azure`, `Custom` |

### Service/Volume/Mount Types

| Type | Fields |
|------|--------|
| `ServiceConfig` | `enabled`, `version`, `port`, `extra` (IndexMap) |
| `VolumeMount` | `target` |
| `MountSpec` | `source`, `target`, `read_only` |
| `McpServerConfig` | `image`, `command`, `env`, `volumes`, `port` |

## Merge Semantics

- **Project always wins**: Any field explicitly set in project config overrides user defaults
- **Raw TOML detection**: Uses raw `toml::Value` parse to distinguish "field absent" (use default) from "field explicitly set to default value" (keep project value)
- **Auto-managed modules skipped**: User defaults cannot inject `ubuntu`, `debian`, `alpine`, `claude-code`, `codex-cli`, or `firewall` modules
- **Base version safety**: `base_version` only merged from user when `base` also comes from user (prevents Alpine version on Ubuntu base)
- **user-setup propagation**: When user-setup params come from user defaults, they propagate back to `image.user` and `image.shell`

## Validation Rules

| Check | Severity | Description |
|-------|----------|-------------|
| Auth for agent type | Warning | Selected agent type should have matching auth config |
| Port conflicts | Error | All enabled services' ports (primary + secondary) must be unique |
| Port range | Error | Custom ports must be 1–65535 |
| Firewall capabilities | Warning | Firewall enabled requires NET_ADMIN in `[runtime].cap_add` |

## OS Version Defaults

| BaseOs | Default Version |
|--------|----------------|
| Ubuntu | 24.04 |
| Debian | bookworm |
| Alpine | 3.21 |

## Implementation Files

| File | Role | Key Exports |
|------|------|-------------|
| `src/config/mod.rs` | Config loading entry points | `load_project_config`, `load_user_config`, `load_effective_config` |
| `src/config/project.rs` | All config type definitions | `ProjectConfig`, `AgentType`, `BaseOs`, `ShellType`, all nested structs/enums |
| `src/config/user.rs` | User config structure | `UserConfig`, `UserDefaults` |
| `src/config/merge.rs` | Merge logic with raw TOML comparison | `merge_configs` |
| `src/config/validate.rs` | Validation rules | `validate_config`, `ValidationWarning`, `default_port_for_service` |

## Invariants and Constraints

1. `[project].name` and `[agent].type` are the only required fields
2. All other fields have sensible defaults via serde
3. `IndexMap` used everywhere instead of `HashMap` to preserve insertion order
4. Config is immutable after loading — never mutated during generation
5. Merge function takes `&mut ProjectConfig` but only fills in absent fields, never overwrites
6. Validation returns warnings (non-fatal) separately from errors (fatal via Result)
7. All enums use lowercase serde deserialization (`#[serde(rename_all = "lowercase")]`)
