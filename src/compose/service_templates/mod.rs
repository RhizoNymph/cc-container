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
        ServiceTemplateInfo {
            name: "postgres",
            description: "PostgreSQL database",
            category: ServiceCategory::Database,
            default_port: 5432,
        },
        ServiceTemplateInfo {
            name: "mysql",
            description: "MySQL database",
            category: ServiceCategory::Database,
            default_port: 3306,
        },
        ServiceTemplateInfo {
            name: "mariadb",
            description: "MariaDB database",
            category: ServiceCategory::Database,
            default_port: 3306,
        },
        ServiceTemplateInfo {
            name: "mongodb",
            description: "MongoDB database",
            category: ServiceCategory::Database,
            default_port: 27017,
        },
        ServiceTemplateInfo {
            name: "cockroachdb",
            description: "CockroachDB database",
            category: ServiceCategory::Database,
            default_port: 26257,
        },
        ServiceTemplateInfo {
            name: "redis",
            description: "Redis in-memory store",
            category: ServiceCategory::Cache,
            default_port: 6379,
        },
        ServiceTemplateInfo {
            name: "memcached",
            description: "Memcached cache",
            category: ServiceCategory::Cache,
            default_port: 11211,
        },
        ServiceTemplateInfo {
            name: "rabbitmq",
            description: "RabbitMQ message broker",
            category: ServiceCategory::Queue,
            default_port: 5672,
        },
        ServiceTemplateInfo {
            name: "kafka",
            description: "Apache Kafka (via Redpanda)",
            category: ServiceCategory::Queue,
            default_port: 9092,
        },
        ServiceTemplateInfo {
            name: "nats",
            description: "NATS messaging",
            category: ServiceCategory::Queue,
            default_port: 4222,
        },
        ServiceTemplateInfo {
            name: "elasticsearch",
            description: "Elasticsearch search engine",
            category: ServiceCategory::Search,
            default_port: 9200,
        },
        ServiceTemplateInfo {
            name: "meilisearch",
            description: "Meilisearch search engine",
            category: ServiceCategory::Search,
            default_port: 7700,
        },
        ServiceTemplateInfo {
            name: "typesense",
            description: "Typesense search engine",
            category: ServiceCategory::Search,
            default_port: 8108,
        },
        ServiceTemplateInfo {
            name: "minio",
            description: "MinIO S3-compatible storage",
            category: ServiceCategory::Storage,
            default_port: 9000,
        },
        ServiceTemplateInfo {
            name: "prometheus",
            description: "Prometheus monitoring",
            category: ServiceCategory::Monitoring,
            default_port: 9090,
        },
        ServiceTemplateInfo {
            name: "grafana",
            description: "Grafana dashboards",
            category: ServiceCategory::Monitoring,
            default_port: 3000,
        },
        ServiceTemplateInfo {
            name: "traefik",
            description: "Traefik reverse proxy",
            category: ServiceCategory::Proxy,
            default_port: 80,
        },
        ServiceTemplateInfo {
            name: "nginx",
            description: "Nginx reverse proxy",
            category: ServiceCategory::Proxy,
            default_port: 80,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::ServiceConfig;
    use indexmap::IndexMap;

    fn default_service_config() -> ServiceConfig {
        ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: IndexMap::new(),
        }
    }

    #[test]
    fn test_list_all_returns_18_services() {
        let all = list_all();
        assert_eq!(all.len(), 18);
    }

    #[test]
    fn test_list_all_unique_names() {
        let all = list_all();
        let names: Vec<&str> = all.iter().map(|s| s.name).collect();
        let mut unique_names = names.clone();
        unique_names.sort();
        unique_names.dedup();
        assert_eq!(
            names.len(),
            unique_names.len(),
            "Service names must be unique"
        );
    }

    #[test]
    fn test_list_all_categories() {
        let all = list_all();
        let categories: Vec<ServiceCategory> = all.iter().map(|s| s.category).collect();

        assert!(categories.contains(&ServiceCategory::Database));
        assert!(categories.contains(&ServiceCategory::Cache));
        assert!(categories.contains(&ServiceCategory::Queue));
        assert!(categories.contains(&ServiceCategory::Search));
        assert!(categories.contains(&ServiceCategory::Storage));
        assert!(categories.contains(&ServiceCategory::Monitoring));
        assert!(categories.contains(&ServiceCategory::Proxy));
    }

    #[test]
    fn test_list_all_database_count() {
        let all = list_all();
        let db_count = all
            .iter()
            .filter(|s| s.category == ServiceCategory::Database)
            .count();
        assert_eq!(db_count, 5);
    }

    #[test]
    fn test_list_all_cache_count() {
        let all = list_all();
        let count = all
            .iter()
            .filter(|s| s.category == ServiceCategory::Cache)
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_list_all_queue_count() {
        let all = list_all();
        let count = all
            .iter()
            .filter(|s| s.category == ServiceCategory::Queue)
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_list_all_search_count() {
        let all = list_all();
        let count = all
            .iter()
            .filter(|s| s.category == ServiceCategory::Search)
            .count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_build_service_unknown() {
        let config = default_service_config();
        let result = build_service("nonexistent", &config);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ServiceNotFound(name) => assert_eq!(name, "nonexistent"),
            e => panic!("Expected ServiceNotFound, got: {:?}", e),
        }
    }

    #[test]
    fn test_build_service_all_known_services() {
        let config = default_service_config();
        let names = [
            "postgres",
            "mysql",
            "mariadb",
            "mongodb",
            "cockroachdb",
            "redis",
            "memcached",
            "rabbitmq",
            "kafka",
            "nats",
            "elasticsearch",
            "meilisearch",
            "typesense",
            "minio",
            "prometheus",
            "grafana",
            "traefik",
            "nginx",
        ];

        for name in &names {
            let result = build_service(name, &config);
            assert!(result.is_ok(), "Failed to build service: {}", name);
        }
    }

    #[test]
    fn test_build_service_returns_service_and_env() {
        let config = default_service_config();
        let (svc, env) = build_service("postgres", &config).unwrap();

        assert!(svc.image.is_some());
        assert!(env.contains_key("DATABASE_URL"));
    }

    #[test]
    fn test_service_category_display() {
        assert_eq!(format!("{}", ServiceCategory::Database), "database");
        assert_eq!(format!("{}", ServiceCategory::Cache), "cache");
        assert_eq!(format!("{}", ServiceCategory::Queue), "queue");
        assert_eq!(format!("{}", ServiceCategory::Search), "search");
        assert_eq!(format!("{}", ServiceCategory::Storage), "storage");
        assert_eq!(format!("{}", ServiceCategory::Monitoring), "monitoring");
        assert_eq!(format!("{}", ServiceCategory::Proxy), "proxy");
    }

    #[test]
    fn test_default_ports_match_well_known() {
        let all = list_all();
        let find = |name: &str| all.iter().find(|s| s.name == name).unwrap().default_port;

        assert_eq!(find("postgres"), 5432);
        assert_eq!(find("mysql"), 3306);
        assert_eq!(find("mariadb"), 3306);
        assert_eq!(find("mongodb"), 27017);
        assert_eq!(find("cockroachdb"), 26257);
        assert_eq!(find("redis"), 6379);
        assert_eq!(find("memcached"), 11211);
        assert_eq!(find("rabbitmq"), 5672);
        assert_eq!(find("kafka"), 9092);
        assert_eq!(find("nats"), 4222);
        assert_eq!(find("elasticsearch"), 9200);
        assert_eq!(find("meilisearch"), 7700);
        assert_eq!(find("typesense"), 8108);
        assert_eq!(find("minio"), 9000);
        assert_eq!(find("prometheus"), 9090);
        assert_eq!(find("grafana"), 3000);
        assert_eq!(find("traefik"), 80);
        assert_eq!(find("nginx"), 80);
    }

    #[test]
    fn test_proxy_services_return_empty_agent_env() {
        let config = default_service_config();

        let (_, traefik_env) = build_service("traefik", &config).unwrap();
        assert!(traefik_env.is_empty());

        let (_, nginx_env) = build_service("nginx", &config).unwrap();
        assert!(nginx_env.is_empty());
    }
}
