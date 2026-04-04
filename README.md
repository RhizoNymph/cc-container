# cc-container

A Rust CLI that generates containerized AI coding agent environments. Define your stack in a single `cc-container.toml` and get a complete Dockerfile, docker-compose.yml, firewall rules, env templates, MCP configs, and Helm charts.

Supports **Claude Code** and **OpenAI Codex** agents with 19 built-in modules, 18 infrastructure services, and configurable network firewall policies.

## Quick Start

```bash
# Install
cargo install --path .

# Initialize a new project (interactive wizard)
cc-container init

# Generate all output files
cc-container generate

# Build and run
docker compose up --build
```

The `init` wizard walks you through selecting an agent, base OS, languages, tools, services, and firewall settings, then writes a `cc-container.toml` for you. For non-interactive use:

```bash
cc-container init --template claude --no-interactive
cc-container init --template codex --agent codex --no-interactive
cc-container init --template minimal --no-interactive
```

## What Gets Generated

| File | Description |
|------|-------------|
| `Dockerfile` | Multi-stage image with OS, languages, tools, and agent runtime |
| `docker-compose.yml` | Full stack: agent container, infrastructure services, MCP sidecars |
| `init-firewall.sh` | iptables script for domain-based network whitelisting |
| `.env.example` | Environment variable template (auth keys, service credentials) |
| `.mcp.json` | Model Context Protocol server configuration |
| `helm/` | Kubernetes Helm chart (optional) |

## Configuration

Everything is driven by `cc-container.toml`:

```toml
[project]
name = "my-project"

[agent]
type = "claude"  # claude | codex | both

[image]
base = "ubuntu"
base_version = "24.04"
tag = "my-project:latest"
user = "dev"
shell = "bash"

[auth.claude]
method = "api-key"  # api-key | oauth | bedrock | bedrock-api-key | vertex | proxy

[image.modules]
node = { version = "22" }
python = { version = "3.12" }
rust = { version = "stable" }
git = {}

[services.postgres]
enabled = true
version = "15"
port = 5432

[services.redis]
enabled = true

[firewall]
enabled = true
allowed_domains = ["api.github.com", "registry.npmjs.org"]
allowed_cidrs = ["10.0.0.0/8"]
allow_ssh = true
allow_dns = true

[workspace]
mount_path = "/workspace"

[environment]
NODE_ENV = "development"

[runtime]
memory_limit = "4g"
cpu_limit = "2"
cap_add = ["NET_ADMIN", "NET_RAW"]
```

A user-level config at `~/.config/cc-container/config.toml` is merged with the project config (project takes precedence).

## Built-in Modules

Modules are composable Dockerfile building blocks with dependency resolution and parameterization.

| Category | Modules |
|----------|---------|
| **Base** | `ubuntu`, `debian`, `alpine` |
| **Languages** | `node`, `python`, `rust`, `go`, `java`, `ruby`, `dotnet`, `zig`, `cpp` |
| **Tools** | `git`, `build-essential`, `docker-cli` |
| **Agents** | `claude-code`, `codex-cli` |
| **Security** | `user-setup`, `firewall` |

```bash
# List all available modules
cc-container module list

# Show module details (parameters, dependencies)
cc-container module info python

# Add a module to your config
cc-container module add go --with version=1.23

# Scaffold a custom module
cc-container module create --name my-tool
```

## Infrastructure Services

18 pre-configured service templates with health checks, volumes, and environment variables.

| Category | Services |
|----------|----------|
| **Database** | `postgres`, `mysql`, `mariadb`, `mongodb`, `cockroachdb` |
| **Cache** | `redis`, `memcached` |
| **Queue** | `rabbitmq`, `kafka`, `nats` |
| **Search** | `elasticsearch`, `meilisearch`, `typesense` |
| **Storage** | `minio` |
| **Monitoring** | `prometheus`, `grafana` |
| **Proxy** | `traefik`, `nginx` |

```bash
cc-container service list
cc-container service add postgres redis --with version=16
```

## MCP Servers

Configure [Model Context Protocol](https://modelcontextprotocol.io/) server sidecars that run alongside your agent:

```bash
cc-container mcp add my-server \
  --image ghcr.io/org/mcp-server:latest \
  --command serve \
  --env API_KEY=\$MCP_API_KEY \
  --volume /data:/data
```

Or in `cc-container.toml`:

```toml
[mcp.my-server]
image = "ghcr.io/org/mcp-server:latest"
command = ["serve"]
env = ["API_KEY=$MCP_API_KEY"]
volumes = ["/data:/data"]
port = 3000
```

## Network Firewall

When `[firewall] enabled = true`, cc-container generates an iptables script that implements a default-deny outbound policy with domain whitelisting. Agent-specific domains (Anthropic API, npm registry, etc.) are automatically included.

Requires `NET_ADMIN` and `NET_RAW` capabilities (added automatically when firewall is enabled).

## Helm Chart Generation

Generate a Kubernetes Helm chart from your config:

```bash
cc-container generate --only helm
```

```toml
[helm]
image_registry = "ghcr.io/myorg"
image_repository = "my-project"
image_tag = "latest"
namespace = "default"
storage_class = "fast-ssd"
default_pvc_size = "50Gi"
ingress_host = "my-app.example.com"
ingress_class = "nginx"
```

## CLI Reference

```
cc-container [OPTIONS] <COMMAND>

Commands:
  init          Interactive project initialization
  generate      Generate Dockerfile, compose, firewall, env, MCP, Helm
  module        Manage Dockerfile modules (list, info, add, remove, create)
  service       Manage compose service templates (list, info, add, remove)
  mcp           Manage MCP servers (list, add, remove)
  config        Configuration management (show, validate, set, get, edit)
  doctor        Diagnose common issues
  completions   Generate shell completions (bash, zsh, fish, powershell, elvish)

Options:
  --target-dir <PATH>   Project directory (default: cwd)
  --config <PATH>       Config file path
  -v...                 Verbosity (warn → info → debug → trace)
  -q, --quiet           Suppress non-error output
  --color <MODE>        auto | always | never
```

### Generate Options

```bash
cc-container generate                        # Generate all files
cc-container generate --only dockerfile      # Just the Dockerfile
cc-container generate --only compose,env     # Compose + env template
cc-container generate --dry-run              # Preview without writing
cc-container generate --diff                 # Show diff against existing files
cc-container generate --output ./out         # Custom output directory
```

## Building from Source

```bash
git clone https://github.com/RhizoNymph/cc-container.git
cd cc-container
cargo build --release
```

Requires Rust 2024 edition (1.85+).

## License

MIT
