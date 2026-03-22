pub mod cache;
pub mod database;
pub mod monitoring;
pub mod proxy;
pub mod queue;
pub mod search;
pub mod storage;

use crate::config::project::ServiceConfig;
use crate::error::{Error, Result};
use docker_compose_types as dct;
use indexmap::IndexMap;

/// Category of a service template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceCategory {
    Database,
    Cache,
    Queue,
    Search,
    Storage,
    Monitoring,
    Proxy,
}

impl std::fmt::Display for ServiceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database => write!(f, "database"),
            Self::Cache => write!(f, "cache"),
            Self::Queue => write!(f, "queue"),
            Self::Search => write!(f, "search"),
            Self::Storage => write!(f, "storage"),
            Self::Monitoring => write!(f, "monitoring"),
            Self::Proxy => write!(f, "proxy"),
        }
    }
}

/// Information about a service template.
pub struct ServiceTemplateInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub category: ServiceCategory,
    pub default_port: u16,
}

/// Build a compose service from a named service template.
pub fn build_service(
    name: &str,
    config: &ServiceConfig,
) -> Result<(dct::Service, IndexMap<String, String>)> {
    match name {
        "postgres" => database::postgres(config),
        "mysql" => database::mysql(config),
        "mariadb" => database::mariadb(config),
        "mongodb" => database::mongodb(config),
        "cockroachdb" => database::cockroachdb(config),
        "redis" => cache::redis(config),
        "memcached" => cache::memcached(config),
        "rabbitmq" => queue::rabbitmq(config),
        "kafka" => queue::kafka(config),
        "nats" => queue::nats(config),
        "elasticsearch" => search::elasticsearch(config),
        "meilisearch" => search::meilisearch(config),
        "typesense" => search::typesense(config),
        "minio" => storage::minio(config),
        "prometheus" => monitoring::prometheus(config),
        "grafana" => monitoring::grafana(config),
        "traefik" => proxy::traefik(config),
        "nginx" => proxy::nginx(config),
        _ => Err(Error::ServiceNotFound(name.to_string())),
    }
}

/// List all available service templates.
pub fn list_all() -> Vec<ServiceTemplateInfo> {
    vec![
        ServiceTemplateInfo { name: "postgres", description: "PostgreSQL database", category: ServiceCategory::Database, default_port: 5432 },
        ServiceTemplateInfo { name: "mysql", description: "MySQL database", category: ServiceCategory::Database, default_port: 3306 },
        ServiceTemplateInfo { name: "mariadb", description: "MariaDB database", category: ServiceCategory::Database, default_port: 3306 },
        ServiceTemplateInfo { name: "mongodb", description: "MongoDB database", category: ServiceCategory::Database, default_port: 27017 },
        ServiceTemplateInfo { name: "cockroachdb", description: "CockroachDB database", category: ServiceCategory::Database, default_port: 26257 },
        ServiceTemplateInfo { name: "redis", description: "Redis in-memory store", category: ServiceCategory::Cache, default_port: 6379 },
        ServiceTemplateInfo { name: "memcached", description: "Memcached cache", category: ServiceCategory::Cache, default_port: 11211 },
        ServiceTemplateInfo { name: "rabbitmq", description: "RabbitMQ message broker", category: ServiceCategory::Queue, default_port: 5672 },
        ServiceTemplateInfo { name: "kafka", description: "Apache Kafka (via Redpanda)", category: ServiceCategory::Queue, default_port: 9092 },
        ServiceTemplateInfo { name: "nats", description: "NATS messaging", category: ServiceCategory::Queue, default_port: 4222 },
        ServiceTemplateInfo { name: "elasticsearch", description: "Elasticsearch search engine", category: ServiceCategory::Search, default_port: 9200 },
        ServiceTemplateInfo { name: "meilisearch", description: "Meilisearch search engine", category: ServiceCategory::Search, default_port: 7700 },
        ServiceTemplateInfo { name: "typesense", description: "Typesense search engine", category: ServiceCategory::Search, default_port: 8108 },
        ServiceTemplateInfo { name: "minio", description: "MinIO S3-compatible storage", category: ServiceCategory::Storage, default_port: 9000 },
        ServiceTemplateInfo { name: "prometheus", description: "Prometheus monitoring", category: ServiceCategory::Monitoring, default_port: 9090 },
        ServiceTemplateInfo { name: "grafana", description: "Grafana dashboards", category: ServiceCategory::Monitoring, default_port: 3000 },
        ServiceTemplateInfo { name: "traefik", description: "Traefik reverse proxy", category: ServiceCategory::Proxy, default_port: 80 },
        ServiceTemplateInfo { name: "nginx", description: "Nginx reverse proxy", category: ServiceCategory::Proxy, default_port: 80 },
    ]
}
