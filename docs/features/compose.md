# Compose Generation

Generates typed `docker-compose.yml` files using the `docker-compose-types` crate, with 18 infrastructure service templates and agent container construction.

## Scope

**In scope:**
- Docker Compose file generation from ProjectConfig
- 18 infrastructure service templates across 7 categories
- Agent service construction (single or dual Claude/Codex)
- Environment variable aggregation (auth + infrastructure + user-defined)
- Volume collection (workspace, named, auth, additional mounts)
- MCP server sidecar services in compose
- `.env.example` file generation
- DATABASE_URL aggregation logic for multiple databases

**Not in scope:**
- Docker image building or execution
- Service template content customization beyond config options
- Docker networking configuration (uses default compose network)

## Data/Control Flow

```
compose::generator::generate(config: &ProjectConfig)
    │
    ├── 1. Collect enabled infrastructure services
    │     └── For each service in config.services where enabled=true:
    │           build_service(name, config) → (dct::Service, agent_env_vars)
    │
    ├── 2. Aggregate environment variables
    │     ├── If 1 database → DATABASE_URL
    │     ├── If multiple databases → POSTGRES_URL, MYSQL_URL, etc. + warning
    │     └── Non-database services → their specific env vars (REDIS_URL, etc.)
    │
    ├── 3. Build MCP server services
    │     └── For each mcp config: create mcp-{name} compose service
    │
    ├── 4. Build agent service(s)
    │     ├── agent_service::build(config, agent_type, infra_env, depends_on, dockerfile)
    │     │     ├── Collect auth requirements (env vars + volumes)
    │     │     ├── Inject infrastructure env vars
    │     │     ├── Inject user environment vars
    │     │     ├── Build volume list (workspace + named + auth + additional)
    │     │     ├── Set dependency conditions (service_healthy for all infra)
    │     │     └── Apply runtime constraints (CPU, memory, caps, security)
    │     ├── Single agent: service named "agent"
    │     └── Dual agents: "agent-claude" + "agent-codex"
    │
    ├── 5. Collect top-level named volumes
    │     ├── From service mounts (e.g., pgdata-devdb:/var/lib/postgresql/data)
    │     ├── From user-defined config.volumes
    │     └── Exclude path-based volumes (starting with ., /, ~, or containing $)
    │
    └── 6. Return dct::Compose → serialized to YAML

compose::env::generate_env_example(config: &ProjectConfig)
    │
    ├── Auth credential placeholders
    ├── Service passwords and connection details
    └── MCP server credentials
```

## Service Templates (18 total)

### Database (5)

| Service | Image | Default Port | Agent Env | Healthcheck | Secondary Ports |
|---------|-------|-------------|-----------|-------------|-----------------|
| postgres | `postgres:{version}` | 5432 | `DATABASE_URL` | `pg_isready` | — |
| mysql | `mysql:{version}` | 3306 | `DATABASE_URL` | `mysqladmin ping` | — |
| mariadb | `mariadb:{version}` | 3306 | `DATABASE_URL` | `mariadb-admin ping` | — |
| mongodb | `mongo:{version}` | 27017 | `MONGODB_URL` | `mongosh ping` | — |
| cockroachdb | `cockroachdb/cockroach:{version}` | 26257 | `DATABASE_URL` | HTTP health | 8080 (UI) |

**Common database config options:** `version`, `port`, `database` (default: "devdb"), `user` (default: "dev"), `password_env` (default: SERVICE_PASSWORD), `root_password_env` (MySQL/MariaDB only).

### Cache (2)

| Service | Image | Default Port | Agent Env | Healthcheck |
|---------|-------|-------------|-----------|-------------|
| redis | `redis:{version}` | 6379 | `REDIS_URL` | `redis-cli ping` |
| memcached | `memcached:{version}` | 11211 | `MEMCACHED_URL` | `echo stats \| nc` |

### Queue (3)

| Service | Image | Default Port | Agent Env | Healthcheck | Secondary Ports |
|---------|-------|-------------|-----------|-------------|-----------------|
| rabbitmq | `rabbitmq:{version}` | 5672 | `RABBITMQ_URL` | `rabbitmq-diagnostics ping` | `management_port` (15672) |
| kafka | `redpandadata/redpanda:{version}` | 9092 | `KAFKA_BROKERS` | `rpk cluster health` | `schema_registry_port` (8081) |
| nats | `nats:{version}` | 4222 | `NATS_URL` | HTTP health | `monitoring_port` (8222) |

### Search (3)

| Service | Image | Default Port | Agent Env | Healthcheck |
|---------|-------|-------------|-----------|-------------|
| elasticsearch | `docker.elastic.co/elasticsearch/elasticsearch:{version}` | 9200 | `ELASTICSEARCH_URL` | Cluster health |
| meilisearch | `getmeili/meilisearch:{version}` | 7700 | `MEILISEARCH_URL` | HTTP `/health` |
| typesense | `typesense/typesense:{version}` | 8108 | `TYPESENSE_URL` | HTTP `/health` |

### Storage (1)

| Service | Image | Default Port | Agent Env | Healthcheck | Secondary Ports |
|---------|-------|-------------|-----------|-------------|-----------------|
| minio | `minio/minio:{version}` | 9000 | `S3_ENDPOINT`, `S3_ACCESS_KEY_ID`, `S3_SECRET_ACCESS_KEY` | `mc ready local` | `console_port` (9001) |

### Monitoring (2)

| Service | Image | Default Port | Agent Env | Healthcheck |
|---------|-------|-------------|-----------|-------------|
| prometheus | `prom/prometheus:{version}` | 9090 | `PROMETHEUS_URL` | HTTP `/-/healthy` |
| grafana | `grafana/grafana:{version}` | 3000 | `GRAFANA_URL` | HTTP `/api/health` |

### Proxy (2)

| Service | Image | Default Port | Agent Env | Healthcheck |
|---------|-------|-------------|-----------|-------------|
| traefik | `traefik:{version}` | 80 | (none) | `traefik healthcheck --ping` |
| nginx | `nginx:{version}` | 80 | (none) | `curl http://localhost:80/` |

## Agent Service Construction

The `agent_service::build()` function constructs the agent container with:

**Build step:**
- Simple (`BuildStep::Simple(".")`) when using default `Dockerfile`
- Advanced (`BuildStep::Advanced`) with `dockerfile` field for `Dockerfile.claude` or `Dockerfile.codex`

**Environment variables** (in collection order):
1. Auth requirements from `auth::requirements()`
2. Infrastructure service connection strings (DATABASE_URL, REDIS_URL, etc.)
3. User-defined vars from `config.environment.vars`

**Volumes** (in order):
1. Workspace mount: `./:config.workspace.mount_path`
2. Named volumes: `{name}:{volume.target}`
3. Auth volumes (read-only or read-write based on auth method)
4. Additional workspace mounts with optional read-only flag

**Dependencies:**
- Conditional `service_healthy` dependency on all enabled infrastructure services

**Fixed defaults:**
- `stdin_open: true`, `tty: true` (interactive mode)
- `restart: "unless-stopped"`
- `env_file: ".env"` (or custom list from config)
- `working_dir: config.workspace.mount_path`

**Runtime constraints:**
- CPU limits via `deploy.resources.limits.cpus`
- Memory limits via `mem_limit`
- Shared memory via `shm_size`
- Capabilities via `cap_add`, `cap_drop`
- Security options via `security_opt`

## DATABASE_URL Aggregation

When multiple database services are enabled:
- **1 database**: Agent gets `DATABASE_URL` pointing to that service
- **Multiple databases**: Agent gets service-specific names (`POSTGRES_URL`, `MYSQL_URL`, etc.) and a warning is printed

## Public API

| Function | Signature | Description |
|----------|-----------|-------------|
| `generate()` | `(config: &ProjectConfig) -> Result<dct::Compose>` | Generate complete compose structure |
| `build_service()` | `(name, config) -> Result<(dct::Service, IndexMap<String, String>)>` | Build individual service + agent env |
| `list_all()` | `-> Vec<ServiceTemplateInfo>` | List metadata for all 18 service templates |
| `agent_service::build()` | `(config, agent_type, infra_env, depends_on, dockerfile) -> dct::Service` | Build agent container definition |
| `env::generate_env_example()` | `(config: &ProjectConfig) -> String` | Generate `.env.example` template |

## Implementation Files

| File | Role | Key Exports |
|------|------|-------------|
| `src/compose/mod.rs` | Public exports | `ServiceCategory`, `ServiceTemplateInfo`, `generate`, `build_service`, `list_all` |
| `src/compose/generator.rs` | Main compose orchestration | `generate()` |
| `src/compose/agent_service.rs` | Agent container builder | `build()`, `get_auth_requirements()` |
| `src/compose/env.rs` | .env.example generation | `generate_env_example()` |
| `src/compose/service_templates/mod.rs` | Service dispatcher | `build_service()`, `list_all()` |
| `src/compose/service_templates/database.rs` | postgres, mysql, mariadb, mongodb, cockroachdb | Individual builder functions |
| `src/compose/service_templates/cache.rs` | redis, memcached | Individual builder functions |
| `src/compose/service_templates/queue.rs` | rabbitmq, kafka, nats | Individual builder functions |
| `src/compose/service_templates/search.rs` | elasticsearch, meilisearch, typesense | Individual builder functions |
| `src/compose/service_templates/storage.rs` | minio | Builder function |
| `src/compose/service_templates/monitoring.rs` | prometheus, grafana | Individual builder functions |
| `src/compose/service_templates/proxy.rs` | traefik, nginx | Individual builder functions |

## Invariants and Constraints

1. **All services include healthchecks**: Every service template defines a healthcheck with `retries: 5` and `restart: "unless-stopped"`.
2. **Named volumes auto-registered**: Volumes referenced in service mounts (e.g., `pgdata-devdb:/var/lib/postgresql/data`) are automatically added to compose top-level `volumes:`.
3. **Path volumes excluded**: Volume strings starting with `.`, `/`, `~`, or containing `$` are not registered as top-level named volumes.
4. **Typed output**: All compose YAML is generated through `docker-compose-types` structs, never raw strings.
5. **Env var references use `${}`**: Service env vars reference `.env` file values via `${VAR}` syntax.
6. **Proxy services return empty env maps**: Traefik and Nginx don't inject environment variables into the agent container.
7. **CockroachDB always insecure**: Uses `--insecure` mode, always exposes both port 26257 and UI port 8080.
8. **Custom secondary ports validated**: Ports like `management_port`, `console_port`, etc. must be 1–65535 or produce `Error::InvalidPort`.
9. **MCP services always restart**: MCP sidecar services have `restart: "unless-stopped"` but no healthchecks.
10. **Agent service naming**: Single agent = "agent", dual agents = "agent-claude" + "agent-codex".
