# Config

Handles loading, merging, and validating project and user configuration.

## Config Files

- **Project config**: `cc-container.toml` in the project directory
- **User config**: `~/.config/cc-container/config.toml` for global defaults

User config values are merged into project config (project takes precedence).

## Validation

Checks for: port conflicts between services, valid module references, required fields, valid auth methods, valid CIDR/domain formats in firewall config.

## Implementation Files

- `src/config/project.rs` — `ProjectConfig` struct and all nested config types
- `src/config/user.rs` — `UserConfig` for global settings
- `src/config/merge.rs` — Merge user config into project config
- `src/config/validate.rs` — Config validation rules
- `src/config/mod.rs` — Config loading functions
