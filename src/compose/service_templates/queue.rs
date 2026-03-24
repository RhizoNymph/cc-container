use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn rabbitmq(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("3-management-alpine");
    let host_port = config.port.unwrap_or(5672);

    let svc = dct::Service {
        image: Some(format!("rabbitmq:{version}")),
        ports: dct::Ports::Short(vec![
            format!("{host_port}:5672"),
            "15672:15672".to_string(),
        ]),
        volumes: vec![dct::Volumes::Simple("rabbitmqdata:/var/lib/rabbitmq".to_string())],
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

    // Use Redpanda as a Kafka-compatible broker (simpler single-node setup)
    let svc = dct::Service {
        image: Some(format!("redpandadata/redpanda:{version}")),
        command: Some(dct::Command::Simple(
            "redpanda start --smp 1 --memory 512M --overprovisioned --kafka-addr 0.0.0.0:9092".to_string()
        )),
        ports: dct::Ports::Short(vec![
            format!("{host_port}:9092"),
            "8081:8081".to_string(),
        ]),
        volumes: vec![dct::Volumes::Simple("redpandadata:/var/lib/redpanda/data".to_string())],
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

    let agent_env = IndexMap::from([(
        "KAFKA_BROKERS".to_string(),
        "kafka:9092".to_string(),
    )]);

    Ok((svc, agent_env))
}

pub fn nats(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let host_port = config.port.unwrap_or(4222);

    let svc = dct::Service {
        image: Some(format!("nats:{version}")),
        command: Some(dct::Command::Simple("--jetstream --http_port 8222".to_string())),
        ports: dct::Ports::Short(vec![
            format!("{host_port}:4222"),
            "8222:8222".to_string(),
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

    let agent_env = IndexMap::from([(
        "NATS_URL".to_string(),
        "nats://nats:4222".to_string(),
    )]);

    Ok((svc, agent_env))
}
