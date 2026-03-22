use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn rabbitmq(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("3-management-alpine");
    let port = config.port.unwrap_or(5672);

    let svc = dct::Service {
        image: Some(format!("rabbitmq:{version}")),
        ports: dct::Ports::Short(vec![
            format!("{port}:5672"),
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
        format!("amqp://guest:guest@rabbitmq:{port}"),
    )]);

    Ok((svc, agent_env))
}

pub fn kafka(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(9092);

    // Use Redpanda as a Kafka-compatible broker (simpler single-node setup)
    let svc = dct::Service {
        image: Some(format!("redpandadata/redpanda:{version}")),
        command: Some(dct::Command::Simple(format!(
            "redpanda start --smp 1 --memory 512M --overprovisioned --kafka-addr 0.0.0.0:{port}"
        ))),
        ports: dct::Ports::Short(vec![
            format!("{port}:9092"),
            "8081:8081".to_string(),
        ]),
        volumes: vec![dct::Volumes::Simple("redpandadata:/var/lib/redpanda/data".to_string())],
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "KAFKA_BROKERS".to_string(),
        format!("kafka:{port}"),
    )]);

    Ok((svc, agent_env))
}

pub fn nats(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(4222);

    let svc = dct::Service {
        image: Some(format!("nats:{version}")),
        command: Some(dct::Command::Simple("--jetstream".to_string())),
        ports: dct::Ports::Short(vec![
            format!("{port}:4222"),
            "8222:8222".to_string(),
        ]),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "NATS_URL".to_string(),
        format!("nats://nats:{port}"),
    )]);

    Ok((svc, agent_env))
}
