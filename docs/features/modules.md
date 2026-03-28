# Module System

Extensible system for building Dockerfiles from composable, dependency-aware template modules.

## Scope

**In scope:**
- Module definition format (TOML metadata + Jinja2 templates)
- Built-in module registry (19 modules across 5 categories, embedded via `include_str!`)
- User-defined module loading from filesystem
- Dependency resolution via topological sort (petgraph)
- Parameter validation (type checking, allowed values)
- Dockerfile rendering via minijinja
- Custom module injection (pre_agent / post_agent)
- Auto-addition of required modules (base image, agent, user-setup, firewall)

**Not in scope:**
- Module template content (the `.j2` files themselves)
- Docker image building or execution
- Module version compatibility checking

## Data/Control Flow

```
DockerfileGenerator::generate(config, agent_type)
    │
    ├── 1. Extract module configuration from ProjectConfig
    │
    ├── 2. Auto-add required modules:
    │     ├── Base image module (ubuntu/debian/alpine based on config.image.base)
    │     ├── user-setup module (with username and shell from config.image)
    │     ├── Agent module (claude-code and/or codex-cli based on agent_type)
    │     └── firewall module (if config.firewall.enabled)
    │
    ├── 3. Extract custom module snippets (pre_agent, post_agent)
    │     └── Remove "custom" from module list
    │
    ├── 4. ModuleResolver::resolve(enabled_modules)
    │     ├── Collect all enabled module names
    │     ├── Auto-add modules from `requires` dependencies (if missing)
    │     ├── Validate no conflicts exist between enabled modules
    │     ├── Build petgraph DiGraph:
    │     │     ├── Each module = node
    │     │     ├── `requires` = edge (required → requirer)
    │     │     └── `after` = edge (predecessor → successor)
    │     ├── Run petgraph::algo::toposort()
    │     └── Return ordered Vec<String>
    │
    ├── 5. For each module in resolved order:
    │     ├── Look up ModuleEntry in registry
    │     ├── merge_with_defaults(module_name, definition, user_params)
    │     │     ├── Validate parameter types match declared ParamType
    │     │     └── Validate values are in allowed_values (if declared)
    │     ├── Render Jinja2 template with context:
    │     │     ├── params: merged defaults + user overrides
    │     │     ├── base_os: config.image.base
    │     │     └── image: { user, shell }
    │     ├── Insert pre_agent snippet before first agent module
    │     └── Append rendered output to Dockerfile
    │
    ├── 6. Insert post_agent snippet after agent modules
    │
    └── 7. Append USER and WORKDIR directives → return Dockerfile string
```

## Module Definition Format

```toml
[module]
name = "example"
category = "lang"          # base | lang | tool | agent | security | custom
description = "..."
version = "1.0.0"

[module.parameters]
version = { type = "string", default = "20", description = "Node.js version" }
flag = { type = "bool", default = false, description = "Enable feature" }
count = { type = "int", default = 4, description = "Thread count" }
extras = { type = "list", default = [], description = "Extra packages" }
variant = { type = "string", default = "lts", allowed_values = ["lts", "current"] }

[module.dependencies]
requires = ["build-essential"]   # auto-added if missing from config
conflicts = ["other-module"]     # prevents co-installation
after = ["base"]                 # ordering constraint (no auto-add)

[module.metadata]
env_vars = ["NODE_PATH"]         # environment variables set by module
exposed_ports = [3000]           # ports exposed by module
volumes = ["/data"]              # volumes used by module
```

## Types

### Core Types

| Type | Description |
|------|-------------|
| `ModuleDefinition` | Wrapper containing `ModuleMeta` (deserialized from TOML) |
| `ModuleMeta` | Name, category, description, version, parameters, dependencies, metadata |
| `ModuleCategory` | Enum: `Base`, `Lang`, `Tool`, `Agent`, `Security`, `Custom` |
| `ParameterDef` | Parameter type, default, description, optional allowed_values |
| `ParamType` | Enum: `String`, `Bool`, `Int`, `List` |
| `ModuleDependencies` | `requires`, `conflicts`, `after` (all `Vec<String>`) |
| `ModuleMetadata` | `env_vars`, `exposed_ports`, `volumes` |

### Registry and Pipeline Types

| Type | Description |
|------|-------------|
| `ModuleEntry` | Registry entry: `ModuleDefinition` + `String` template |
| `BuiltinModule` | Built-in: `ModuleDefinition` + `&'static str` template |
| `ModuleRegistry` | `IndexMap<String, ModuleEntry>` of all available modules |
| `ModuleResolver<'a>` | Holds `&ModuleRegistry`, performs dependency resolution |
| `DockerfileGenerator<'a>` | Holds `&ModuleRegistry`, renders complete Dockerfiles |

## Built-in Modules (19 total)

### Base (3) — mutually exclusive

| Module | Description | Key Parameters |
|--------|-------------|----------------|
| `ubuntu` | Ubuntu base image | `version` (default: "24.04") |
| `debian` | Debian base image | `version` (default: "bookworm") |
| `alpine` | Alpine base image | `version` (default: "3.21") |

### Lang (9)

| Module | Description | Key Parameters | Dependencies |
|--------|-------------|----------------|--------------|
| `node` | Node.js | `version` (default: "22") | — |
| `python` | Python | `version` | — |
| `rust` | Rust toolchain | — | — |
| `go` | Go | — | — |
| `java` | Java runtime | — | — |
| `ruby` | Ruby | — | — |
| `dotnet` | .NET framework | — | — |
| `zig` | Zig | — | — |
| `cpp` | C++ compiler | — | — |

### Tool (3)

| Module | Description |
|--------|-------------|
| `git` | Git version control |
| `build-essential` | Build tools (gcc, make, etc.) |
| `docker-cli` | Docker CLI client |

### Agent (2)

| Module | Description | Dependencies |
|--------|-------------|--------------|
| `claude-code` | Claude Code CLI | requires: `node` |
| `codex-cli` | OpenAI Codex CLI | requires: `node` |

### Security (2)

| Module | Description | Key Parameters |
|--------|-------------|----------------|
| `user-setup` | Non-root user creation | `username`, `shell` |
| `firewall` | iptables configuration | — |

## Parameter Validation

When user-provided parameters are merged with module defaults (`merge_with_defaults`):

1. **Type checking**: Each parameter value must match the declared `ParamType`. A mismatch produces `Error::InvalidParameter`.
2. **Allowed values**: If `allowed_values` is declared, the provided value must be in the list. Values outside the list produce `Error::InvalidParameter`.
3. **Default fallback**: Missing parameters use the module's declared default value.

## Custom Modules

The `[modules.custom]` config section supports raw Dockerfile injection:

- `pre_agent`: Injected before the first agent module in the Dockerfile
- `post_agent`: Injected after all agent modules

If `[modules.custom]` is present but has neither `pre_agent` nor `post_agent`, the renderer returns an error to prevent silent misconfiguration.

## Shell Path Resolution

The `user-setup` module resolves shell paths dynamically at build time using `$(command -v <shell>)` rather than hardcoding `/bin/<shell>`. This handles shells like `zsh` that install at `/usr/bin/zsh` on Debian/Ubuntu.

## Public API

| Function | Signature | Description |
|----------|-----------|-------------|
| `ModuleRegistry::new()` | `-> Self` | Create registry with all 19 built-ins loaded |
| `ModuleRegistry::load_user_modules()` | `(&mut self, dir: &Path) -> Result<()>` | Load user modules from directory |
| `ModuleRegistry::get()` | `(&self, name: &str) -> Option<&ModuleEntry>` | Look up module by name |
| `ModuleRegistry::all()` | `(&self) -> &IndexMap<String, ModuleEntry>` | All registered modules |
| `ModuleRegistry::contains()` | `(&self, name: &str) -> bool` | Check if module exists |
| `ModuleResolver::resolve()` | `(&self, enabled: &IndexMap) -> Result<Vec<String>>` | Resolve dependencies, return sorted names |
| `DockerfileGenerator::generate()` | `(&self, config, agent_type) -> Result<String>` | Generate complete Dockerfile |
| `load_all()` | `-> Vec<BuiltinModule>` | Load all built-in module definitions + templates |

## Implementation Files

| File | Role | Key Exports |
|------|------|-------------|
| `src/module/mod.rs` | Module entry point | Re-exports `ModuleRegistry`, `DockerfileGenerator` |
| `src/module/definition.rs` | Type definitions | `ModuleDefinition`, `ModuleMeta`, `ModuleCategory`, `ParameterDef`, `ParamType`, `ModuleDependencies` |
| `src/module/registry.rs` | Module loading/caching | `ModuleRegistry`, `ModuleEntry` |
| `src/module/resolver.rs` | Dependency resolution | `ModuleResolver` (petgraph topological sort) |
| `src/module/renderer.rs` | Dockerfile rendering | `DockerfileGenerator`, `merge_with_defaults` |
| `src/module/builtin/mod.rs` | Built-in module loader | `BuiltinModule`, `load_all()` |
| `src/module/builtin/<category>/<name>/` | Module assets | `<name>.toml` + `<name>.dockerfile.j2` pairs |

## Invariants and Constraints

1. **Topological ordering is mandatory**: Modules are always rendered in dependency-resolved order. Circular dependencies produce `Error::CircularDependency`.
2. **Base modules are mutually exclusive**: `ubuntu`, `debian`, and `alpine` all declare conflicts with each other. Only one base can be active.
3. **Agent modules require node**: Both `claude-code` and `codex-cli` declare `requires: ["node"]`, which is auto-added if missing.
4. **Auto-managed modules**: Base image, user-setup, agent, and firewall modules are auto-added by the renderer based on config — they should not be manually configured by users.
5. **Module names must be unique**: Registry uses `IndexMap<String, ModuleEntry>` keyed by name. User modules can override built-ins by matching the name.
6. **Template context is fixed**: Templates receive exactly `params`, `base_os`, and `image` (user, shell). No other config data is available in templates.
7. **All built-ins are embedded**: Module TOML + templates are compiled into the binary via `include_str!`. Changes require recompilation.
8. **IndexMap preserves order**: Module insertion order is preserved throughout the pipeline.
