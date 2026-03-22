use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn prometheus(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(9090);

    let svc = dct::Service {
        image: Some(format!("prom/prometheus:{version}")),
        ports: dct::Ports::Short(vec![format!("{port}:9090")]),
        volumes: vec![dct::Volumes::Simple("promdata:/prometheus".to_string())],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "wget -q --spider http://localhost:9090/-/healthy || exit 1".to_string(),
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
        "PROMETHEUS_URL".to_string(),
        "http://prometheus:9090".to_string(),
    )]);

    Ok((svc, agent_env))
}

pub fn grafana(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(3000);

    let svc = dct::Service {
        image: Some(format!("grafana/grafana:{version}")),
        ports: dct::Ports::Short(vec![format!("{port}:3000")]),
        volumes: vec![dct::Volumes::Simple("grafanadata:/var/lib/grafana".to_string())],
        environment: dct::Environment::KvPair(IndexMap::from([
            ("GF_SECURITY_ADMIN_PASSWORD".to_string(), Some(dct::SingleValue::String("${GRAFANA_PASSWORD:-admin}".to_string()))),
        ])),
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "wget -q --spider http://localhost:3000/api/health || exit 1".to_string(),
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
        "GRAFANA_URL".to_string(),
        "http://grafana:3000".to_string(),
    )]);

    Ok((svc, agent_env))
}
