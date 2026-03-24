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
    fn test_prometheus_defaults() {
        let config = default_config();
        let (svc, env) = prometheus(&config).unwrap();

        assert_eq!(svc.image, Some("prom/prometheus:latest".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["9090:9090"]),
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "promdata:/prometheus"),
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("9090/-/healthy"));
        }

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        assert_eq!(env["PROMETHEUS_URL"], "http://prometheus:9090");
    }

    #[test]
    fn test_prometheus_custom_port() {
        let config = ServiceConfig {
            port: Some(19090),
            ..default_config()
        };
        let (svc, _) = prometheus(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["19090:9090"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_prometheus_custom_version() {
        let config = ServiceConfig {
            version: Some("v2.48.0".to_string()),
            ..default_config()
        };
        let (svc, _) = prometheus(&config).unwrap();
        assert_eq!(svc.image, Some("prom/prometheus:v2.48.0".to_string()));
    }

    #[test]
    fn test_grafana_defaults() {
        let config = default_config();
        let (svc, env) = grafana(&config).unwrap();

        assert_eq!(svc.image, Some("grafana/grafana:latest".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["3000:3000"]),
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "grafanadata:/var/lib/grafana"),
            _ => panic!("Expected simple volume"),
        }

        if let dct::Environment::KvPair(e) = &svc.environment {
            assert!(e.contains_key("GF_SECURITY_ADMIN_PASSWORD"));
        } else {
            panic!("Expected KvPair environment");
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("3000/api/health"));
        }

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        assert_eq!(env["GRAFANA_URL"], "http://grafana:3000");
    }

    #[test]
    fn test_grafana_custom_port() {
        let config = ServiceConfig {
            port: Some(13000),
            ..default_config()
        };
        let (svc, _) = grafana(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["13000:3000"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_monitoring_services_have_healthchecks() {
        let config = default_config();
        for builder in [prometheus, grafana] {
            let (svc, _) = builder(&config).unwrap();
            assert!(svc.healthcheck.is_some());
            let hc = svc.healthcheck.unwrap();
            assert_eq!(hc.retries, 5);
        }
    }

    #[test]
    fn test_monitoring_services_have_volumes() {
        let config = default_config();
        for builder in [prometheus, grafana] {
            let (svc, _) = builder(&config).unwrap();
            assert!(!svc.volumes.is_empty());
        }
    }
}
