use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn rabbitmq(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("3-management-alpine");
    let host_port = config.port.unwrap_or(5672);

    let mgmt_port: u16 = match config
        .extra
        .get("management_port")
        .and_then(|v| v.as_integer())
    {
        Some(v) if (1..=65535).contains(&v) => v as u16,
        Some(v) => {
            return Err(crate::error::Error::InvalidPort {
                value: v,
                context: "rabbitmq extra.management_port".to_string(),
            });
        }
        None => 15672,
    };

    let svc = dct::Service {
        image: Some(format!("rabbitmq:{version}")),
        ports: dct::Ports::Short(vec![
            format!("{host_port}:5672"),
            format!("{mgmt_port}:15672"),
        ]),
        volumes: vec![dct::Volumes::Simple(
            "rabbitmqdata:/var/lib/rabbitmq".to_string(),
        )],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "rabbitmq-diagnostics -q ping".to_string(),
            )),
            interval: Some("10s".to_string()),
            timeout: Some("5s".to_string()),
            retries: 5,
            ..Default::default()
        }),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "RABBITMQ_URL".to_string(),
        "amqp://guest:guest@rabbitmq:5672".to_string(),
    )]);

    Ok((svc, agent_env))
}

pub fn kafka(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let host_port = config.port.unwrap_or(9092);

    let registry_port: u16 = match config
        .extra
        .get("schema_registry_port")
        .and_then(|v| v.as_integer())
    {
        Some(v) if (1..=65535).contains(&v) => v as u16,
        Some(v) => {
            return Err(crate::error::Error::InvalidPort {
                value: v,
                context: "kafka extra.schema_registry_port".to_string(),
            });
        }
        None => 8081,
    };

    // Use Redpanda as a Kafka-compatible broker (simpler single-node setup)
    let svc = dct::Service {
        image: Some(format!("redpandadata/redpanda:{version}")),
        command: Some(dct::Command::Simple(
            "redpanda start --smp 1 --memory 512M --overprovisioned --kafka-addr 0.0.0.0:9092"
                .to_string(),
        )),
        ports: dct::Ports::Short(vec![
            format!("{host_port}:9092"),
            format!("{registry_port}:8081"),
        ]),
        volumes: vec![dct::Volumes::Simple(
            "redpandadata:/var/lib/redpanda/data".to_string(),
        )],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "rpk cluster health | grep -q 'HEALTHY' || exit 1".to_string(),
            )),
            interval: Some("10s".to_string()),
            timeout: Some("5s".to_string()),
            retries: 5,
            start_period: Some("30s".to_string()),
            ..Default::default()
        }),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([("KAFKA_BROKERS".to_string(), "kafka:9092".to_string())]);

    Ok((svc, agent_env))
}

pub fn nats(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let host_port = config.port.unwrap_or(4222);

    let monitoring_port: u16 = match config
        .extra
        .get("monitoring_port")
        .and_then(|v| v.as_integer())
    {
        Some(v) if (1..=65535).contains(&v) => v as u16,
        Some(v) => {
            return Err(crate::error::Error::InvalidPort {
                value: v,
                context: "nats extra.monitoring_port".to_string(),
            });
        }
        None => 8222,
    };

    let svc = dct::Service {
        image: Some(format!("nats:{version}")),
        command: Some(dct::Command::Simple(
            "--jetstream --http_port 8222".to_string(),
        )),
        ports: dct::Ports::Short(vec![
            format!("{host_port}:4222"),
            format!("{monitoring_port}:8222"),
        ]),
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "wget -q --spider http://localhost:8222/healthz || exit 1".to_string(),
            )),
            interval: Some("10s".to_string()),
            timeout: Some("5s".to_string()),
            retries: 5,
            ..Default::default()
        }),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([("NATS_URL".to_string(), "nats://nats:4222".to_string())]);

    Ok((svc, agent_env))
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

    #[test]
    fn test_rabbitmq_defaults() {
        let config = default_config();
        let (svc, env) = rabbitmq(&config).unwrap();

        assert_eq!(svc.image, Some("rabbitmq:3-management-alpine".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert_eq!(ports.len(), 2);
                assert!(ports.contains(&"5672:5672".to_string()));
                assert!(ports.contains(&"15672:15672".to_string()));
            }
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "rabbitmqdata:/var/lib/rabbitmq"),
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("rabbitmq-diagnostics"));
        }

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        assert_eq!(env["RABBITMQ_URL"], "amqp://guest:guest@rabbitmq:5672");
    }

    #[test]
    fn test_rabbitmq_custom_port() {
        let config = ServiceConfig {
            port: Some(15672),
            ..default_config()
        };
        let (svc, _) = rabbitmq(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"15672:5672".to_string()));
                assert!(ports.contains(&"15672:15672".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_kafka_defaults() {
        let config = default_config();
        let (svc, env) = kafka(&config).unwrap();

        assert_eq!(svc.image, Some("redpandadata/redpanda:latest".to_string()));

        match &svc.command {
            Some(dct::Command::Simple(cmd)) => assert!(cmd.contains("redpanda start")),
            _ => panic!("Expected simple command"),
        }

        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert_eq!(ports.len(), 2);
                assert!(ports.contains(&"9092:9092".to_string()));
                assert!(ports.contains(&"8081:8081".to_string()));
            }
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "redpandadata:/var/lib/redpanda/data"),
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("rpk cluster health"));
        }

        assert_eq!(env["KAFKA_BROKERS"], "kafka:9092");
    }

    #[test]
    fn test_kafka_custom_port() {
        let config = ServiceConfig {
            port: Some(19092),
            ..default_config()
        };
        let (svc, _) = kafka(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"19092:9092".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_kafka_custom_version() {
        let config = ServiceConfig {
            version: Some("v23.3.2".to_string()),
            ..default_config()
        };
        let (svc, _) = kafka(&config).unwrap();
        assert_eq!(svc.image, Some("redpandadata/redpanda:v23.3.2".to_string()));
    }

    #[test]
    fn test_nats_defaults() {
        let config = default_config();
        let (svc, env) = nats(&config).unwrap();

        assert_eq!(svc.image, Some("nats:latest".to_string()));

        match &svc.command {
            Some(dct::Command::Simple(cmd)) => {
                assert!(cmd.contains("--jetstream"));
                assert!(cmd.contains("--http_port 8222"));
            }
            _ => panic!("Expected simple command"),
        }

        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert_eq!(ports.len(), 2);
                assert!(ports.contains(&"4222:4222".to_string()));
                assert!(ports.contains(&"8222:8222".to_string()));
            }
            _ => panic!("Expected short ports"),
        }

        assert!(svc.volumes.is_empty());

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("healthz"));
        }

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        assert_eq!(env["NATS_URL"], "nats://nats:4222");
    }

    #[test]
    fn test_nats_custom_port() {
        let config = ServiceConfig {
            port: Some(14222),
            ..default_config()
        };
        let (svc, _) = nats(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"14222:4222".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    // ───────────────────── Configurable secondary ports ─────────────────────

    #[test]
    fn test_rabbitmq_custom_management_port() {
        let mut extra = IndexMap::new();
        extra.insert("management_port".to_string(), toml::Value::Integer(25672));
        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, _) = rabbitmq(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"25672:15672".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_kafka_custom_schema_registry_port() {
        let mut extra = IndexMap::new();
        extra.insert(
            "schema_registry_port".to_string(),
            toml::Value::Integer(18081),
        );
        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, _) = kafka(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"18081:8081".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_nats_custom_monitoring_port() {
        let mut extra = IndexMap::new();
        extra.insert("monitoring_port".to_string(), toml::Value::Integer(18222));
        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, _) = nats(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"18222:8222".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_rabbitmq_invalid_management_port() {
        let mut extra = IndexMap::new();
        extra.insert("management_port".to_string(), toml::Value::Integer(99999));
        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let err = rabbitmq(&config).unwrap_err();
        match err {
            crate::error::Error::InvalidPort { value, context } => {
                assert_eq!(value, 99999);
                assert!(context.contains("management_port"));
            }
            other => panic!("Expected InvalidPort, got: {:?}", other),
        }
    }

    #[test]
    fn test_minio_invalid_console_port() {
        let mut extra = IndexMap::new();
        extra.insert("console_port".to_string(), toml::Value::Integer(99999));
        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let err = super::super::storage::minio(&config).unwrap_err();
        match err {
            crate::error::Error::InvalidPort { value, context } => {
                assert_eq!(value, 99999);
                assert!(context.contains("console_port"));
            }
            other => panic!("Expected InvalidPort, got: {:?}", other),
        }
    }

    #[test]
    fn test_queue_services_have_healthchecks() {
        let config = default_config();
        for builder in [rabbitmq, kafka, nats] {
            let (svc, _) = builder(&config).unwrap();
            assert!(svc.healthcheck.is_some());
        }
    }

    #[test]
    fn test_queue_services_have_restart_policy() {
        let config = default_config();
        for builder in [rabbitmq, kafka, nats] {
            let (svc, _) = builder(&config).unwrap();
            assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        }
    }
}
