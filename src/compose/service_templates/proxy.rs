use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn traefik(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(80);

    let svc = dct::Service {
        image: Some(format!("traefik:{version}")),
        command: Some(dct::Command::Simple(
            "--api.insecure=true --providers.docker=true --ping=true".to_string(),
        )),
        ports: dct::Ports::Short(vec![
            format!("{port}:80"),
            "8080:8080".to_string(),
        ]),
        volumes: vec![dct::Volumes::Simple(
            "/var/run/docker.sock:/var/run/docker.sock:ro".to_string(),
        )],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "traefik healthcheck --ping || exit 1".to_string(),
            )),
            interval: Some("10s".to_string()),
            timeout: Some("5s".to_string()),
            retries: 5,
            ..Default::default()
        }),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    Ok((svc, IndexMap::new()))
}

pub fn nginx(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("alpine");
    let port = config.port.unwrap_or(80);

    let svc = dct::Service {
        image: Some(format!("nginx:{version}")),
        ports: dct::Ports::Short(vec![format!("{port}:80")]),
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "curl -f http://localhost:80/ || exit 1".to_string(),
            )),
            interval: Some("10s".to_string()),
            timeout: Some("5s".to_string()),
            retries: 5,
            ..Default::default()
        }),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    Ok((svc, IndexMap::new()))
}
