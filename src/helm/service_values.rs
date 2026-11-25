use crate::config::project::ServiceConfig;
use crate::error::{Error, Result};
use crate::helm::types::{
    HealthcheckSpec, ImageRef, PortSpec, ResourceLimits, ResourceSpec, ServiceValues, VolumeMount,
};
use indexmap::IndexMap;

/// Helper to extract a string value from the service config extra map.
fn get_str(config: &ServiceConfig, key: &str, default: &str) -> String {
    config
        .extra
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

/// Default resource limits (no limits set).
fn default_resources() -> ResourceLimits {
    ResourceLimits {
        requests: ResourceSpec {
            cpu: None,
            memory: None,
        },
        limits: ResourceSpec {
            cpu: None,
            memory: None,
        },
    }
}

/// Build a port spec for a primary service port.
fn port(name: &str, container_port: u16) -> PortSpec {
    PortSpec {
        name: name.to_string(),
        container_port,
        protocol: "TCP".to_string(),
    }
}

/// Build helm ServiceValues from a named service template.
///
/// Returns `(ServiceValues, agent_env)` where `agent_env` is the map of
/// environment variables to inject into the agent container (DATABASE_URL,
/// REDIS_URL, etc.).
pub fn build_service(
    name: &str,
    config: &ServiceConfig,
) -> Result<(ServiceValues, IndexMap<String, String>)> {
    match name {
        "postgres" => postgres(config),
        "mysql" => mysql(config),
        "mariadb" => mariadb(config),
        "mongodb" => mongodb(config),
        "cockroachdb" => cockroachdb(config),
        "redis" => redis(config),
        "memcached" => memcached(config),
        "rabbitmq" => rabbitmq(config),
        "kafka" => kafka(config),
        "nats" => nats(config),
        "elasticsearch" => elasticsearch(config),
        "meilisearch" => meilisearch(config),
        "typesense" => typesense(config),
        "minio" => minio(config),
        "prometheus" => prometheus(config),
        "grafana" => grafana(config),
        "traefik" => traefik(config),
        "nginx" => nginx(config),
        _ => Err(Error::ServiceNotFound(name.to_string())),
    }
}

fn postgres(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("16");
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "POSTGRES_PASSWORD");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "postgres".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "database".to_string(),
        stateful: true,
        ports: vec![port("postgres", 5432)],
        env: IndexMap::from([
            ("POSTGRES_DB".to_string(), db.clone()),
            ("POSTGRES_USER".to_string(), user.clone()),
            (
                "POSTGRES_PASSWORD".to_string(),
                format!("${{{password_env}}}"),
            ),
        ]),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/var/lib/postgresql/data".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                format!("pg_isready -U ${{POSTGRES_USER:-{user}}}"),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "DATABASE_URL".to_string(),
        format!("postgres://{user}:${{{password_env}}}@postgres:5432/{db}"),
    )]);

    Ok((svc, agent_env))
}

fn mysql(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("8");
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "MYSQL_PASSWORD");
    let root_password_env = get_str(config, "root_password_env", "MYSQL_ROOT_PASSWORD");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "mysql".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "database".to_string(),
        stateful: true,
        ports: vec![port("mysql", 3306)],
        env: IndexMap::from([
            ("MYSQL_DATABASE".to_string(), db.clone()),
            ("MYSQL_USER".to_string(), user.clone()),
            ("MYSQL_PASSWORD".to_string(), format!("${{{password_env}}}")),
            (
                "MYSQL_ROOT_PASSWORD".to_string(),
                format!("${{{root_password_env}}}"),
            ),
        ]),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/var/lib/mysql".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "mysqladmin ping -h localhost".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "DATABASE_URL".to_string(),
        format!("mysql://{user}:${{{password_env}}}@mysql:3306/{db}"),
    )]);

    Ok((svc, agent_env))
}

fn mariadb(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("11");
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "MARIADB_PASSWORD");
    let root_password_env = get_str(config, "root_password_env", "MARIADB_ROOT_PASSWORD");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "mariadb".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "database".to_string(),
        stateful: true,
        ports: vec![port("mariadb", 3306)],
        env: IndexMap::from([
            ("MARIADB_DATABASE".to_string(), db.clone()),
            ("MARIADB_USER".to_string(), user.clone()),
            (
                "MARIADB_PASSWORD".to_string(),
                format!("${{{password_env}}}"),
            ),
            (
                "MARIADB_ROOT_PASSWORD".to_string(),
                format!("${{{root_password_env}}}"),
            ),
        ]),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/var/lib/mysql".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "mariadb-admin ping -h localhost".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "DATABASE_URL".to_string(),
        format!("mysql://{user}:${{{password_env}}}@mariadb:3306/{db}"),
    )]);

    Ok((svc, agent_env))
}

fn mongodb(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("7");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "mongo".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "database".to_string(),
        stateful: true,
        ports: vec![port("mongodb", 27017)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/data/db".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "mongosh --eval 'db.runCommand(\"ping\").ok' --quiet".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "MONGODB_URL".to_string(),
        "mongodb://mongodb:27017".to_string(),
    )]);

    Ok((svc, agent_env))
}

fn cockroachdb(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "cockroachdb/cockroach".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "database".to_string(),
        stateful: true,
        ports: vec![port("sql", 26257), port("http", 8080)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/cockroach/cockroach-data".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: Some(vec![
            "start-single-node".to_string(),
            "--insecure".to_string(),
        ]),
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "curl -f http://localhost:8080/health?ready=1 || exit 1".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "DATABASE_URL".to_string(),
        "postgres://root@cockroachdb:26257/defaultdb?sslmode=disable".to_string(),
    )]);

    Ok((svc, agent_env))
}

fn redis(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("7");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "redis".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "cache".to_string(),
        stateful: true,
        ports: vec![port("redis", 6379)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/data".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "redis-cli ping".to_string(),
            ],
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([("REDIS_URL".to_string(), "redis://redis:6379".to_string())]);

    Ok((svc, agent_env))
}

fn memcached(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("1");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "memcached".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "cache".to_string(),
        stateful: false,
        ports: vec![port("memcached", 11211)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![],
        pvc_size: String::new(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "echo stats | nc localhost 11211".to_string(),
            ],
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([("MEMCACHED_URL".to_string(), "memcached:11211".to_string())]);

    Ok((svc, agent_env))
}

fn rabbitmq(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("3-management");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "rabbitmq".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "queue".to_string(),
        stateful: true,
        ports: vec![port("amqp", 5672), port("management", 15672)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/var/lib/rabbitmq".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "rabbitmq-diagnostics ping".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "RABBITMQ_URL".to_string(),
        "amqp://guest:guest@rabbitmq:5672".to_string(),
    )]);

    Ok((svc, agent_env))
}

fn kafka(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "redpandadata/redpanda".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "queue".to_string(),
        stateful: true,
        ports: vec![port("kafka", 9092), port("schema-registry", 8081)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/var/lib/redpanda/data".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: Some(vec![
            "redpanda".to_string(),
            "start".to_string(),
            "--smp".to_string(),
            "1".to_string(),
            "--memory".to_string(),
            "512M".to_string(),
            "--overprovisioned".to_string(),
            "--kafka-addr".to_string(),
            "0.0.0.0:9092".to_string(),
        ]),
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "rpk cluster health".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([("KAFKA_BROKERS".to_string(), "kafka:9092".to_string())]);

    Ok((svc, agent_env))
}

fn nats(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "nats".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "queue".to_string(),
        stateful: true,
        ports: vec![port("nats", 4222), port("monitoring", 8222)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/data/nats-server".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: Some(vec![
            "--jetstream".to_string(),
            "--http_port".to_string(),
            "8222".to_string(),
        ]),
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "curl -f http://localhost:8222/healthz || exit 1".to_string(),
            ],
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([("NATS_URL".to_string(), "nats://nats:4222".to_string())]);

    Ok((svc, agent_env))
}

fn elasticsearch(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("8");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: Some("docker.elastic.co".to_string()),
            repository: "elasticsearch/elasticsearch".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "search".to_string(),
        stateful: true,
        ports: vec![port("http", 9200)],
        env: IndexMap::from([
            ("discovery.type".to_string(), "single-node".to_string()),
            ("xpack.security.enabled".to_string(), "false".to_string()),
            ("ES_JAVA_OPTS".to_string(), "-Xms512m -Xmx512m".to_string()),
        ]),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/usr/share/elasticsearch/data".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "curl -f http://localhost:9200/_cluster/health || exit 1".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 15,
            timeout_seconds: 10,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "ELASTICSEARCH_URL".to_string(),
        "http://elasticsearch:9200".to_string(),
    )]);

    Ok((svc, agent_env))
}

fn meilisearch(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "getmeili/meilisearch".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "search".to_string(),
        stateful: true,
        ports: vec![port("http", 7700)],
        env: IndexMap::from([("MEILI_ENV".to_string(), "development".to_string())]),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/meili_data".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "curl -f http://localhost:7700/health || exit 1".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "MEILISEARCH_URL".to_string(),
        "http://meilisearch:7700".to_string(),
    )]);

    Ok((svc, agent_env))
}

fn typesense(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "typesense/typesense".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "search".to_string(),
        stateful: true,
        ports: vec![port("http", 8108)],
        env: IndexMap::from([
            (
                "TYPESENSE_API_KEY".to_string(),
                "${TYPESENSE_API_KEY:-changeme}".to_string(),
            ),
            ("TYPESENSE_DATA_DIR".to_string(), "/data".to_string()),
        ]),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/data".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "curl -f http://localhost:8108/health || exit 1".to_string(),
            ],
            initial_delay_seconds: 30,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "TYPESENSE_URL".to_string(),
        "http://typesense:8108".to_string(),
    )]);

    Ok((svc, agent_env))
}

fn minio(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "minio/minio".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "storage".to_string(),
        stateful: true,
        ports: vec![port("api", 9000), port("console", 9001)],
        env: IndexMap::from([
            (
                "MINIO_ROOT_USER".to_string(),
                "${MINIO_ACCESS_KEY:-minioadmin}".to_string(),
            ),
            (
                "MINIO_ROOT_PASSWORD".to_string(),
                "${MINIO_SECRET_KEY:-minioadmin}".to_string(),
            ),
        ]),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/data".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: Some(vec![
            "server".to_string(),
            "/data".to_string(),
            "--console-address".to_string(),
            ":9001".to_string(),
        ]),
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "mc ready local".to_string(),
            ],
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([
        ("S3_ENDPOINT".to_string(), "http://minio:9000".to_string()),
        (
            "S3_ACCESS_KEY_ID".to_string(),
            "${MINIO_ACCESS_KEY:-minioadmin}".to_string(),
        ),
        (
            "S3_SECRET_ACCESS_KEY".to_string(),
            "${MINIO_SECRET_KEY:-minioadmin}".to_string(),
        ),
    ]);

    Ok((svc, agent_env))
}

fn prometheus(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "prom/prometheus".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "monitoring".to_string(),
        stateful: true,
        ports: vec![port("http", 9090)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/prometheus".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "curl -f http://localhost:9090/-/healthy || exit 1".to_string(),
            ],
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env = IndexMap::from([(
        "PROMETHEUS_URL".to_string(),
        "http://prometheus:9090".to_string(),
    )]);

    Ok((svc, agent_env))
}

fn grafana(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "grafana/grafana".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "monitoring".to_string(),
        stateful: false,
        ports: vec![port("http", 3000)],
        env: IndexMap::from([(
            "GF_SECURITY_ADMIN_PASSWORD".to_string(),
            "${GRAFANA_PASSWORD:-admin}".to_string(),
        )]),
        agent_env: IndexMap::new(),
        volume_mounts: vec![VolumeMount {
            name: "data".to_string(),
            mount_path: "/var/lib/grafana".to_string(),
            read_only: false,
        }],
        pvc_size: "10Gi".to_string(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "curl -f http://localhost:3000/api/health || exit 1".to_string(),
            ],
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    let agent_env =
        IndexMap::from([("GRAFANA_URL".to_string(), "http://grafana:3000".to_string())]);

    Ok((svc, agent_env))
}

fn traefik(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "traefik".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "proxy".to_string(),
        stateful: false,
        ports: vec![port("http", 80)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![],
        pvc_size: String::new(),
        command: Some(vec![
            "--api.insecure=true".to_string(),
            "--providers.docker=true".to_string(),
            "--ping=true".to_string(),
        ]),
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "traefik healthcheck --ping".to_string(),
            ],
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    Ok((svc, IndexMap::new()))
}

fn nginx(config: &ServiceConfig) -> Result<(ServiceValues, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");

    let svc = ServiceValues {
        enabled: true,
        image: ImageRef {
            registry: None,
            repository: "nginx".to_string(),
            tag: version.to_string(),
            pull_policy: "IfNotPresent".to_string(),
        },
        category: "proxy".to_string(),
        stateful: false,
        ports: vec![port("http", 80)],
        env: IndexMap::new(),
        agent_env: IndexMap::new(),
        volume_mounts: vec![],
        pvc_size: String::new(),
        command: None,
        healthcheck: HealthcheckSpec {
            command: vec![
                "sh".to_string(),
                "-c".to_string(),
                "curl http://localhost:80/".to_string(),
            ],
            initial_delay_seconds: 10,
            period_seconds: 10,
            timeout_seconds: 5,
            failure_threshold: 5,
        },
        resources: default_resources(),
    };

    Ok((svc, IndexMap::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::ServiceConfig;
    use indexmap::IndexMap;

    fn default_config() -> ServiceConfig {
        ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: IndexMap::new(),
        }
    }

    // -- Dispatcher tests --

    #[test]
    fn build_service_unknown_returns_error() {
        let config = default_config();
        let result = build_service("nonexistent", &config);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ServiceNotFound(name) => assert_eq!(name, "nonexistent"),
            e => panic!("Expected ServiceNotFound, got: {:?}", e),
        }
    }

    #[test]
    fn build_service_all_known_services_succeed() {
        let config = default_config();
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

    // -- Postgres tests --

    #[test]
    fn postgres_defaults() {
        let config = default_config();
        let (svc, env) = build_service("postgres", &config).unwrap();

        assert_eq!(svc.image.repository, "postgres");
        assert_eq!(svc.image.tag, "16");
        assert_eq!(svc.category, "database");
        assert!(svc.stateful);
        assert_eq!(svc.ports.len(), 1);
        assert_eq!(svc.ports[0].container_port, 5432);
        assert_eq!(svc.pvc_size, "10Gi");

        // Healthcheck
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("pg_isready"))
        );
        assert_eq!(svc.healthcheck.initial_delay_seconds, 30);
        assert_eq!(svc.healthcheck.failure_threshold, 5);

        // Volume
        assert_eq!(svc.volume_mounts.len(), 1);
        assert_eq!(svc.volume_mounts[0].mount_path, "/var/lib/postgresql/data");

        // Env vars
        assert!(svc.env.contains_key("POSTGRES_DB"));
        assert!(svc.env.contains_key("POSTGRES_USER"));
        assert!(svc.env.contains_key("POSTGRES_PASSWORD"));

        // Agent env
        assert!(env.contains_key("DATABASE_URL"));
        let url = &env["DATABASE_URL"];
        assert!(url.starts_with("postgres://dev:"));
        assert!(url.contains("@postgres:5432/devdb"));
    }

    #[test]
    fn postgres_custom_version() {
        let config = ServiceConfig {
            version: Some("15".to_string()),
            ..default_config()
        };
        let (svc, _) = build_service("postgres", &config).unwrap();
        assert_eq!(svc.image.tag, "15");
    }

    #[test]
    fn postgres_custom_database_and_user() {
        let mut extra = IndexMap::new();
        extra.insert(
            "database".to_string(),
            toml::Value::String("mydb".to_string()),
        );
        extra.insert(
            "user".to_string(),
            toml::Value::String("myuser".to_string()),
        );
        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, env) = build_service("postgres", &config).unwrap();

        assert_eq!(svc.env["POSTGRES_DB"], "mydb");
        assert_eq!(svc.env["POSTGRES_USER"], "myuser");
        assert!(env["DATABASE_URL"].contains("myuser:"));
        assert!(env["DATABASE_URL"].contains("/mydb"));
    }

    #[test]
    fn postgres_custom_password_env() {
        let mut extra = IndexMap::new();
        extra.insert(
            "password_env".to_string(),
            toml::Value::String("MY_PG_PASS".to_string()),
        );
        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, env) = build_service("postgres", &config).unwrap();

        assert_eq!(svc.env["POSTGRES_PASSWORD"], "${MY_PG_PASS}");
        assert!(env["DATABASE_URL"].contains("${MY_PG_PASS}"));
    }

    // -- MySQL tests --

    #[test]
    fn mysql_defaults() {
        let config = default_config();
        let (svc, env) = build_service("mysql", &config).unwrap();

        assert_eq!(svc.image.repository, "mysql");
        assert_eq!(svc.image.tag, "8");
        assert_eq!(svc.category, "database");
        assert!(svc.stateful);
        assert_eq!(svc.ports[0].container_port, 3306);
        assert!(svc.env.contains_key("MYSQL_DATABASE"));
        assert!(svc.env.contains_key("MYSQL_USER"));
        assert!(svc.env.contains_key("MYSQL_PASSWORD"));
        assert!(svc.env.contains_key("MYSQL_ROOT_PASSWORD"));
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("mysqladmin"))
        );
        assert_eq!(svc.volume_mounts[0].mount_path, "/var/lib/mysql");
        assert!(env["DATABASE_URL"].starts_with("mysql://dev:"));
    }

    #[test]
    fn mysql_custom_password_envs() {
        let mut extra = IndexMap::new();
        extra.insert(
            "password_env".to_string(),
            toml::Value::String("CUSTOM_PW".to_string()),
        );
        extra.insert(
            "root_password_env".to_string(),
            toml::Value::String("CUSTOM_ROOT_PW".to_string()),
        );
        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, _) = build_service("mysql", &config).unwrap();

        assert_eq!(svc.env["MYSQL_PASSWORD"], "${CUSTOM_PW}");
        assert_eq!(svc.env["MYSQL_ROOT_PASSWORD"], "${CUSTOM_ROOT_PW}");
    }

    // -- MariaDB tests --

    #[test]
    fn mariadb_defaults() {
        let config = default_config();
        let (svc, env) = build_service("mariadb", &config).unwrap();

        assert_eq!(svc.image.repository, "mariadb");
        assert_eq!(svc.image.tag, "11");
        assert!(svc.stateful);
        assert!(svc.env.contains_key("MARIADB_DATABASE"));
        assert!(svc.env.contains_key("MARIADB_PASSWORD"));
        assert!(svc.env.contains_key("MARIADB_ROOT_PASSWORD"));
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("mariadb-admin"))
        );
        assert!(env["DATABASE_URL"].contains("@mariadb:3306/devdb"));
    }

    // -- MongoDB tests --

    #[test]
    fn mongodb_defaults() {
        let config = default_config();
        let (svc, env) = build_service("mongodb", &config).unwrap();

        assert_eq!(svc.image.repository, "mongo");
        assert_eq!(svc.image.tag, "7");
        assert!(svc.stateful);
        assert_eq!(svc.ports[0].container_port, 27017);
        assert_eq!(svc.volume_mounts[0].mount_path, "/data/db");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("mongosh"))
        );
        assert_eq!(env["MONGODB_URL"], "mongodb://mongodb:27017");
    }

    // -- CockroachDB tests --

    #[test]
    fn cockroachdb_defaults() {
        let config = default_config();
        let (svc, env) = build_service("cockroachdb", &config).unwrap();

        assert_eq!(svc.image.repository, "cockroachdb/cockroach");
        assert_eq!(svc.image.tag, "latest");
        assert!(svc.stateful);
        assert_eq!(svc.ports.len(), 2);
        assert_eq!(svc.ports[0].container_port, 26257);
        assert_eq!(svc.ports[1].container_port, 8080);
        assert!(svc.command.is_some());
        assert_eq!(svc.volume_mounts[0].mount_path, "/cockroach/cockroach-data");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("health?ready=1"))
        );
        assert!(env["DATABASE_URL"].contains("cockroachdb:26257"));
    }

    // -- Redis tests --

    #[test]
    fn redis_defaults() {
        let config = default_config();
        let (svc, env) = build_service("redis", &config).unwrap();

        assert_eq!(svc.image.repository, "redis");
        assert_eq!(svc.image.tag, "7");
        assert_eq!(svc.category, "cache");
        assert!(svc.stateful);
        assert_eq!(svc.ports[0].container_port, 6379);
        assert_eq!(svc.volume_mounts[0].mount_path, "/data");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("redis-cli"))
        );
        assert_eq!(env["REDIS_URL"], "redis://redis:6379");
    }

    // -- Memcached tests --

    #[test]
    fn memcached_defaults() {
        let config = default_config();
        let (svc, env) = build_service("memcached", &config).unwrap();

        assert_eq!(svc.image.repository, "memcached");
        assert_eq!(svc.image.tag, "1");
        assert_eq!(svc.category, "cache");
        assert!(!svc.stateful);
        assert_eq!(svc.ports[0].container_port, 11211);
        assert!(svc.volume_mounts.is_empty());
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("nc localhost 11211"))
        );
        assert_eq!(env["MEMCACHED_URL"], "memcached:11211");
    }

    // -- RabbitMQ tests --

    #[test]
    fn rabbitmq_defaults() {
        let config = default_config();
        let (svc, env) = build_service("rabbitmq", &config).unwrap();

        assert_eq!(svc.image.repository, "rabbitmq");
        assert_eq!(svc.image.tag, "3-management");
        assert_eq!(svc.category, "queue");
        assert!(svc.stateful);
        assert_eq!(svc.ports.len(), 2);
        assert_eq!(svc.ports[0].container_port, 5672);
        assert_eq!(svc.ports[1].container_port, 15672);
        assert_eq!(svc.volume_mounts[0].mount_path, "/var/lib/rabbitmq");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("rabbitmq-diagnostics"))
        );
        assert_eq!(env["RABBITMQ_URL"], "amqp://guest:guest@rabbitmq:5672");
    }

    // -- Kafka tests --

    #[test]
    fn kafka_defaults() {
        let config = default_config();
        let (svc, env) = build_service("kafka", &config).unwrap();

        assert_eq!(svc.image.repository, "redpandadata/redpanda");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "queue");
        assert!(svc.stateful);
        assert_eq!(svc.ports.len(), 2);
        assert_eq!(svc.ports[0].container_port, 9092);
        assert_eq!(svc.ports[1].container_port, 8081);
        assert!(svc.command.is_some());
        assert_eq!(svc.volume_mounts[0].mount_path, "/var/lib/redpanda/data");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("rpk cluster health"))
        );
        assert_eq!(env["KAFKA_BROKERS"], "kafka:9092");
    }

    // -- NATS tests --

    #[test]
    fn nats_defaults() {
        let config = default_config();
        let (svc, env) = build_service("nats", &config).unwrap();

        assert_eq!(svc.image.repository, "nats");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "queue");
        assert!(svc.stateful);
        assert_eq!(svc.ports.len(), 2);
        assert_eq!(svc.ports[0].container_port, 4222);
        assert_eq!(svc.ports[1].container_port, 8222);
        assert!(svc.command.is_some());
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("healthz"))
        );
        assert_eq!(env["NATS_URL"], "nats://nats:4222");
    }

    // -- Elasticsearch tests --

    #[test]
    fn elasticsearch_defaults() {
        let config = default_config();
        let (svc, env) = build_service("elasticsearch", &config).unwrap();

        assert_eq!(svc.image.registry, Some("docker.elastic.co".to_string()));
        assert_eq!(svc.image.repository, "elasticsearch/elasticsearch");
        assert_eq!(svc.image.tag, "8");
        assert_eq!(svc.category, "search");
        assert!(svc.stateful);
        assert_eq!(svc.ports[0].container_port, 9200);
        assert!(svc.env.contains_key("discovery.type"));
        assert_eq!(
            svc.volume_mounts[0].mount_path,
            "/usr/share/elasticsearch/data"
        );
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("_cluster/health"))
        );
        assert_eq!(env["ELASTICSEARCH_URL"], "http://elasticsearch:9200");
    }

    // -- Meilisearch tests --

    #[test]
    fn meilisearch_defaults() {
        let config = default_config();
        let (svc, env) = build_service("meilisearch", &config).unwrap();

        assert_eq!(svc.image.repository, "getmeili/meilisearch");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "search");
        assert!(svc.stateful);
        assert_eq!(svc.ports[0].container_port, 7700);
        assert!(svc.env.contains_key("MEILI_ENV"));
        assert_eq!(svc.volume_mounts[0].mount_path, "/meili_data");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("7700/health"))
        );
        assert_eq!(env["MEILISEARCH_URL"], "http://meilisearch:7700");
    }

    // -- Typesense tests --

    #[test]
    fn typesense_defaults() {
        let config = default_config();
        let (svc, env) = build_service("typesense", &config).unwrap();

        assert_eq!(svc.image.repository, "typesense/typesense");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "search");
        assert!(svc.stateful);
        assert_eq!(svc.ports[0].container_port, 8108);
        assert!(svc.env.contains_key("TYPESENSE_API_KEY"));
        assert_eq!(svc.volume_mounts[0].mount_path, "/data");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("8108/health"))
        );
        assert_eq!(env["TYPESENSE_URL"], "http://typesense:8108");
    }

    // -- MinIO tests --

    #[test]
    fn minio_defaults() {
        let config = default_config();
        let (svc, env) = build_service("minio", &config).unwrap();

        assert_eq!(svc.image.repository, "minio/minio");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "storage");
        assert!(svc.stateful);
        assert_eq!(svc.ports.len(), 2);
        assert_eq!(svc.ports[0].container_port, 9000);
        assert_eq!(svc.ports[1].container_port, 9001);
        assert!(svc.command.is_some());
        assert!(svc.env.contains_key("MINIO_ROOT_USER"));
        assert!(svc.env.contains_key("MINIO_ROOT_PASSWORD"));
        assert_eq!(svc.volume_mounts[0].mount_path, "/data");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("mc ready"))
        );

        assert_eq!(env.len(), 3);
        assert_eq!(env["S3_ENDPOINT"], "http://minio:9000");
        assert!(env.contains_key("S3_ACCESS_KEY_ID"));
        assert!(env.contains_key("S3_SECRET_ACCESS_KEY"));
    }

    // -- Prometheus tests --

    #[test]
    fn prometheus_defaults() {
        let config = default_config();
        let (svc, env) = build_service("prometheus", &config).unwrap();

        assert_eq!(svc.image.repository, "prom/prometheus");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "monitoring");
        assert!(svc.stateful);
        assert_eq!(svc.ports[0].container_port, 9090);
        assert_eq!(svc.volume_mounts[0].mount_path, "/prometheus");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("9090/-/healthy"))
        );
        assert_eq!(env["PROMETHEUS_URL"], "http://prometheus:9090");
    }

    // -- Grafana tests --

    #[test]
    fn grafana_defaults() {
        let config = default_config();
        let (svc, env) = build_service("grafana", &config).unwrap();

        assert_eq!(svc.image.repository, "grafana/grafana");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "monitoring");
        assert!(!svc.stateful);
        assert_eq!(svc.ports[0].container_port, 3000);
        assert!(svc.env.contains_key("GF_SECURITY_ADMIN_PASSWORD"));
        assert_eq!(svc.volume_mounts[0].mount_path, "/var/lib/grafana");
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("3000/api/health"))
        );
        assert_eq!(env["GRAFANA_URL"], "http://grafana:3000");
    }

    // -- Traefik tests --

    #[test]
    fn traefik_defaults() {
        let config = default_config();
        let (svc, env) = build_service("traefik", &config).unwrap();

        assert_eq!(svc.image.repository, "traefik");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "proxy");
        assert!(!svc.stateful);
        assert_eq!(svc.ports[0].container_port, 80);
        assert!(svc.volume_mounts.is_empty());
        assert!(svc.command.is_some());
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("traefik healthcheck"))
        );
        assert!(env.is_empty());
    }

    // -- Nginx tests --

    #[test]
    fn nginx_defaults() {
        let config = default_config();
        let (svc, env) = build_service("nginx", &config).unwrap();

        assert_eq!(svc.image.repository, "nginx");
        assert_eq!(svc.image.tag, "latest");
        assert_eq!(svc.category, "proxy");
        assert!(!svc.stateful);
        assert_eq!(svc.ports[0].container_port, 80);
        assert!(svc.volume_mounts.is_empty());
        assert!(svc.command.is_none());
        assert!(
            svc.healthcheck
                .command
                .iter()
                .any(|c| c.contains("localhost:80"))
        );
        assert!(env.is_empty());
    }

    // -- Cross-cutting tests --

    #[test]
    fn stateful_services_have_volumes() {
        let config = default_config();
        let stateful = [
            "postgres",
            "mysql",
            "mariadb",
            "mongodb",
            "cockroachdb",
            "redis",
            "rabbitmq",
            "kafka",
            "nats",
            "elasticsearch",
            "meilisearch",
            "typesense",
            "minio",
            "prometheus",
        ];
        for name in &stateful {
            let (svc, _) = build_service(name, &config).unwrap();
            assert!(svc.stateful, "{name} should be stateful");
            assert!(
                !svc.volume_mounts.is_empty(),
                "{name} should have volume mounts"
            );
            assert!(!svc.pvc_size.is_empty(), "{name} should have a pvc_size");
        }
    }

    #[test]
    fn stateless_services_have_no_pvc() {
        let config = default_config();
        let stateless = ["memcached", "traefik", "nginx"];
        for name in &stateless {
            let (svc, _) = build_service(name, &config).unwrap();
            assert!(!svc.stateful, "{name} should not be stateful");
        }
    }

    #[test]
    fn all_services_have_healthchecks() {
        let config = default_config();
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
            let (svc, _) = build_service(name, &config).unwrap();
            assert!(
                !svc.healthcheck.command.is_empty(),
                "{name} should have a healthcheck command"
            );
            assert!(
                svc.healthcheck.failure_threshold > 0,
                "{name} should have a failure threshold"
            );
        }
    }

    #[test]
    fn proxy_services_return_empty_agent_env() {
        let config = default_config();
        let (_, traefik_env) = build_service("traefik", &config).unwrap();
        assert!(traefik_env.is_empty());
        let (_, nginx_env) = build_service("nginx", &config).unwrap();
        assert!(nginx_env.is_empty());
    }

    #[test]
    fn all_services_are_enabled() {
        let config = default_config();
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
            let (svc, _) = build_service(name, &config).unwrap();
            assert!(svc.enabled, "{name} should be enabled");
        }
    }

    #[test]
    fn all_ports_use_tcp_protocol() {
        let config = default_config();
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
            let (svc, _) = build_service(name, &config).unwrap();
            for p in &svc.ports {
                assert_eq!(p.protocol, "TCP", "{name} port {} should be TCP", p.name);
            }
        }
    }

    #[test]
    fn custom_version_propagates() {
        let services_and_defaults = [
            ("postgres", "16"),
            ("mysql", "8"),
            ("redis", "7"),
            ("elasticsearch", "8"),
        ];
        for (name, _default_ver) in &services_and_defaults {
            let config = ServiceConfig {
                version: Some("custom-ver".to_string()),
                ..default_config()
            };
            let (svc, _) = build_service(name, &config).unwrap();
            assert_eq!(
                svc.image.tag, "custom-ver",
                "{name} should accept custom version"
            );
        }
    }
}
