use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn traefik(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(80);
    let dashboard_port = config
        .extra
        .get("dashboard_port")
        .and_then(|v| v.as_integer())
        .map(|v| v as u16)
        .unwrap_or(8080);

    let svc = dct::Service {
        image: Some(format!("traefik:{version}")),
        command: Some(dct::Command::Simple(
            "--api.insecure=true --providers.docker=true --ping=true".to_string(),
        )),
        ports: dct::Ports::Short(vec![format!("{port}:80"), format!("{dashboard_port}:8080")]),
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
                "wget --spider -q http://localhost:80/ || exit 1".to_string(),
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
    fn test_traefik_defaults() {
        let config = default_config();
        let (svc, env) = traefik(&config).unwrap();

        assert_eq!(svc.image, Some("traefik:latest".to_string()));

        match &svc.command {
            Some(dct::Command::Simple(cmd)) => {
                assert!(cmd.contains("--api.insecure=true"));
                assert!(cmd.contains("--providers.docker=true"));
                assert!(cmd.contains("--ping=true"));
            }
            _ => panic!("Expected simple command"),
        }

        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert_eq!(ports.len(), 2);
                assert!(ports.contains(&"80:80".to_string()));
                assert!(ports.contains(&"8080:8080".to_string()));
            }
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => {
                assert_eq!(v, "/var/run/docker.sock:/var/run/docker.sock:ro")
            }
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("traefik healthcheck"));
        }

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        assert!(env.is_empty());
    }

    #[test]
    fn test_traefik_custom_port() {
        let config = ServiceConfig {
            port: Some(8000),
            ..default_config()
        };
        let (svc, _) = traefik(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"8000:80".to_string()));
                assert!(ports.contains(&"8080:8080".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_traefik_custom_version() {
        let config = ServiceConfig {
            version: Some("v3.0".to_string()),
            ..default_config()
        };
        let (svc, _) = traefik(&config).unwrap();
        assert_eq!(svc.image, Some("traefik:v3.0".to_string()));
    }

    #[test]
    fn test_nginx_defaults() {
        let config = default_config();
        let (svc, env) = nginx(&config).unwrap();

        assert_eq!(svc.image, Some("nginx:alpine".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["80:80"]),
            _ => panic!("Expected short ports"),
        }

        assert!(svc.volumes.is_empty());

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("localhost:80"));
        }

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        assert!(env.is_empty());
    }

    #[test]
    fn test_nginx_custom_port() {
        let config = ServiceConfig {
            port: Some(8080),
            ..default_config()
        };
        let (svc, _) = nginx(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["8080:80"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_nginx_custom_version() {
        let config = ServiceConfig {
            version: Some("1.25".to_string()),
            ..default_config()
        };
        let (svc, _) = nginx(&config).unwrap();
        assert_eq!(svc.image, Some("nginx:1.25".to_string()));
    }

    #[test]
    fn test_proxy_services_have_healthchecks() {
        let config = default_config();
        for builder in [traefik, nginx] {
            let (svc, _) = builder(&config).unwrap();
            assert!(svc.healthcheck.is_some());
            let hc = svc.healthcheck.unwrap();
            assert_eq!(hc.retries, 5);
        }
    }

    #[test]
    fn test_proxy_services_return_empty_env() {
        let config = default_config();
        for builder in [traefik, nginx] {
            let (_, env) = builder(&config).unwrap();
            assert!(env.is_empty());
        }
    }
}
