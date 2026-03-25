# Compose Generation

Generates typed `docker-compose.yml` files using the `docker-compose-types` crate.

## How It Works

The generator builds the agent service from the generated Dockerfile, then adds infrastructure service sidecars based on `[services]` config. Each service template defines ports, env vars, health checks, and volumes.

## Built-in Service Templates (16)

| Category   | Services                                        |
|------------|------------------------------------------------|
| Database   | PostgreSQL, MySQL, MariaDB, MongoDB, CockroachDB |
| Cache      | Redis, Memcached                                |
| Queue      | RabbitMQ, Kafka, NATS                           |
| Search     | Elasticsearch, Meilisearch, Typesense           |
| Storage    | MinIO                                           |
| Monitoring | Prometheus, Grafana                             |
| Proxy      | Traefik, Nginx                                  |

## Implementation Files

- `src/compose/generator.rs` — Main compose structure generation
- `src/compose/agent_service.rs` — Agent service definition builder
- `src/compose/env.rs` — Environment variable handling
- `src/compose/service_templates/mod.rs` — Service template registry
- `src/compose/service_templates/{database,cache,queue,search,storage,monitoring,proxy}.rs` — Individual templates
