# cc-container Overview

Overview:
    description:
        CLI tool that generates containerized AI coding agent environments from a TOML config file.
        Produces Dockerfiles (via composable template modules), docker-compose.yml stacks (with typed
        service templates), firewall scripts (iptables), .env templates, and MCP server configs.
        Supports both Claude Code and OpenAI Codex agents, individually or together.

    subsystems:
        - **Config** (`src/config/`): Loads project config (`cc-container.toml`) and user config
          (`~/.config/cc-container/config.toml`), merges them (project takes precedence), and validates
          the result (port conflicts, auth completeness, firewall requirements).

        - **Module System** (`src/module/`): Extensible template-based Dockerfile generation. Each module
          is a TOML metadata file + Jinja2 template pair. 19 built-in modules across 5 categories (base,
          lang, tool, agent, security). Dependencies resolved via petgraph topological sort. Templates
          rendered with minijinja.

        - **Compose Generation** (`src/compose/`): Builds typed docker-compose.yml using the
          `docker-compose-types` crate. 18 infrastructure service templates with health checks, volumes,
          and env vars. Constructs agent service(s) with auth, runtime constraints, and infrastructure
          dependency wiring.

        - **Firewall** (`src/firewall/`): Generates iptables bash scripts from domain/CIDR allowlists.
          Default-deny policy with agent-specific default domains. Validates domains and CIDRs.

        - **Auth** (`src/auth/`): Maps authentication config to environment variables and volume mounts
          for Claude (6 methods) and Codex (4 methods) agent containers.

        - **MCP** (`src/mcp/`): Generates Model Context Protocol server configs in two formats:
          `.mcp.json` (docker run commands for Claude discovery) and compose sidecar services.

        - **CLI & Wizard** (`src/cli/`, `src/wizard/`): Clap-based command tree with 8 commands.
          Interactive init wizard via dialoguer. Commands for generating, managing modules/services/MCP,
          config inspection, and environment diagnostics.

        - **Error Handling** (`src/error.rs`): Centralized error enum with thiserror. 16 typed variants
          covering config, modules, templates, services, ports, and IO.

    data_flow:
        ```
        cc-container.toml + ~/.config/cc-container/config.toml
            │
            ▼
        Config Loading → Merge (project wins) → Validation → ProjectConfig
            │
            ├──▶ Module Registry (19 built-ins via include_str!)
            │       │
            │       ▼
            │    Module Resolver (petgraph toposort)
            │       │
            │       ▼
            │    DockerfileGenerator (minijinja render) → Dockerfile(s)
            │
            ├──▶ Compose Generator
            │       ├── Service Templates → infrastructure services
            │       ├── Auth Requirements → env vars + volumes
            │       ├── Agent Service Builder → agent container(s)
            │       └── MCP Services → sidecar containers
            │       │
            │       ▼
            │    docker-compose.yml (via docker-compose-types)
            │
            ├──▶ Firewall Generator → init-firewall.sh (iptables script)
            │
            ├──▶ Env Generator → .env.example
            │
            └──▶ MCP Config Generator → .mcp.json
        ```

Features Index:
    config:
        description: Project and user config loading, merging, and validation
        entry_points: [config::load_effective_config, config::validate_config]
        depends_on: []
        doc: docs/features/config.md

    modules:
        description: Extensible template-based Dockerfile generation with dependency resolution
        entry_points: [module::ModuleRegistry::new, module::DockerfileGenerator::generate]
        depends_on: [config]
        doc: docs/features/modules.md

    compose:
        description: Typed docker-compose.yml generation with 18 service templates
        entry_points: [compose::generator::generate, compose::env::generate_env_example]
        depends_on: [config, auth, mcp]
        doc: docs/features/compose.md

    firewall:
        description: iptables rule generation from domain/CIDR allowlists
        entry_points: [firewall::generator::generate]
        depends_on: [config]
        doc: docs/features/firewall.md

    auth:
        description: Authentication env var and volume mapping for Claude and Codex
        entry_points: [auth::claude::requirements, auth::codex::requirements]
        depends_on: [config]
        doc: docs/features/auth.md

    mcp:
        description: MCP server sidecar configuration generation
        entry_points: [mcp::config::generate_mcp_json]
        depends_on: [config]
        doc: docs/features/mcp.md

    cli:
        description: CLI command structure, interactive wizard, and environment diagnostics
        entry_points: [cli::Cli::parse, wizard::flow::run]
        depends_on: [config, modules, compose, firewall, auth, mcp]
        doc: docs/features/cli.md

## Source Layout

```
src/
├── main.rs                     # Entry point, tracing setup, command dispatch
├── lib.rs                      # Module declarations and re-exports
├── error.rs                    # Error enum (16 variants via thiserror)
├── config/
│   ├── mod.rs                  # load_project_config, load_user_config, load_effective_config
│   ├── project.rs              # ProjectConfig and all nested config types
│   ├── user.rs                 # UserConfig, UserDefaults
│   ├── merge.rs                # merge_configs (project + user with raw TOML detection)
│   └── validate.rs             # validate_config, port conflict detection
├── module/
│   ├── mod.rs                  # Re-exports ModuleRegistry, DockerfileGenerator
│   ├── definition.rs           # ModuleDefinition, ModuleMeta, ParameterDef, dependencies
│   ├── registry.rs             # ModuleRegistry (loads built-ins, optional user modules)
│   ├── resolver.rs             # ModuleResolver (petgraph topological sort)
│   ├── renderer.rs             # DockerfileGenerator (minijinja template rendering)
│   └── builtin/
│       └── mod.rs              # load_all() for 19 embedded modules
├── compose/
│   ├── mod.rs                  # Public exports, ServiceCategory, ServiceTemplateInfo
│   ├── generator.rs            # generate() — orchestrates full compose output
│   ├── agent_service.rs        # Agent container builder (env, volumes, deps, runtime)
│   ├── env.rs                  # generate_env_example()
│   └── service_templates/
│       ├── mod.rs              # build_service dispatcher, list_all()
│       ├── database.rs         # postgres, mysql, mariadb, mongodb, cockroachdb
│       ├── cache.rs            # redis, memcached
│       ├── queue.rs            # rabbitmq, kafka, nats
│       ├── search.rs           # elasticsearch, meilisearch, typesense
│       ├── storage.rs          # minio
│       ├── monitoring.rs       # prometheus, grafana
│       └── proxy.rs            # traefik, nginx
├── firewall/
│   ├── mod.rs                  # Module exports
│   ├── generator.rs            # generate() — iptables bash script
│   └── domains.rs              # claude_defaults(), codex_defaults(), validation
├── auth/
│   ├── mod.rs                  # AuthRequirements, AuthVolume types
│   ├── claude.rs               # Claude auth (6 methods) → env vars + volumes
│   └── codex.rs                # Codex auth (4 methods) → env vars + volumes
├── mcp/
│   ├── mod.rs                  # Module exports
│   ├── config.rs               # generate_mcp_json() → .mcp.json
│   └── service.rs              # MCP compose service generation
├── cli/
│   ├── mod.rs                  # Cli struct, Commands enum, GlobalArgs, clap tree
│   ├── init.rs                 # init command (templates, interactive, defaults)
│   ├── generate.rs             # generate command (Dockerfile, compose, firewall, env, mcp)
│   ├── module.rs               # module subcommands (list, info, add, remove, create)
│   ├── service.rs              # service subcommands (list, info, add, remove)
│   ├── mcp.rs                  # mcp subcommands (list, add, remove)
│   ├── config_cmd.rs           # config subcommands (show, validate, set, get, edit)
│   └── doctor.rs               # doctor command (env diagnostics)
└── wizard/
    ├── mod.rs                  # Module exports
    ├── flow.rs                 # Interactive init flow (10 prompts)
    └── prompts.rs              # Prompt helpers (dialoguer wrappers)
```
