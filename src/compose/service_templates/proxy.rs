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
            "--api.insecure=true --providers.docker=true".to_string(),
        )),
        ports: dct::Ports::Short(vec![
            format!("{port}:80"),
            "8080:8080".to_string(),
        ]),
        volumes: vec![dct::Volumes::Simple(
            "/var/run/docker.sock:/var/run/docker.sock:ro".to_string(),
        )],
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
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    Ok((svc, IndexMap::new()))
}
