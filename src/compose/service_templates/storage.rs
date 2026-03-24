use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn minio(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let host_port = config.port.unwrap_or(9000);

    let console_port = config
        .extra
        .get("console_port")
        .and_then(|v| v.as_integer())
        .unwrap_or(9001) as u16;

    let svc = dct::Service {
        image: Some(format!("minio/minio:{version}")),
        command: Some(dct::Command::Simple("server /data --console-address :9001".to_string())),
        ports: dct::Ports::Short(vec![
            format!("{host_port}:9000"),
            format!("{console_port}:9001"),
        ]),
        environment: dct::Environment::KvPair(IndexMap::from([
            ("MINIO_ROOT_USER".to_string(), Some(dct::SingleValue::String("${MINIO_ACCESS_KEY:-minioadmin}".to_string()))),
            ("MINIO_ROOT_PASSWORD".to_string(), Some(dct::SingleValue::String("${MINIO_SECRET_KEY:-minioadmin}".to_string()))),
        ])),
        volumes: vec![dct::Volumes::Simple("miniodata:/data".to_string())],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "mc ready local || exit 1".to_string(),
            )),
            interval: Some("10s".to_string()),
            timeout: Some("5s".to_string()),
            retries: 5,
            ..Default::default()
        }),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([
        ("S3_ENDPOINT".to_string(), "http://minio:9000".to_string()),
        ("S3_ACCESS_KEY_ID".to_string(), "${MINIO_ACCESS_KEY:-minioadmin}".to_string()),
        ("S3_SECRET_ACCESS_KEY".to_string(), "${MINIO_SECRET_KEY:-minioadmin}".to_string()),
    ]);

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
    fn test_minio_defaults() {
        let config = default_config();
        let (svc, env) = minio(&config).unwrap();

        assert_eq!(svc.image, Some("minio/minio:latest".to_string()));

        match &svc.command {
            Some(dct::Command::Simple(cmd)) => {
                assert!(cmd.contains("server /data"));
                assert!(cmd.contains("--console-address :9001"));
            }
            _ => panic!("Expected simple command"),
        }

        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert_eq!(ports.len(), 2);
                assert!(ports.contains(&"9000:9000".to_string()));
                assert!(ports.contains(&"9001:9001".to_string()));
            }
            _ => panic!("Expected short ports"),
        }

        if let dct::Environment::KvPair(e) = &svc.environment {
            assert!(e.contains_key("MINIO_ROOT_USER"));
            assert!(e.contains_key("MINIO_ROOT_PASSWORD"));
        } else {
            panic!("Expected KvPair environment");
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "miniodata:/data"),
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("mc ready local"));
        }

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));

        assert_eq!(env["S3_ENDPOINT"], "http://minio:9000");
        assert!(env.contains_key("S3_ACCESS_KEY_ID"));
        assert!(env.contains_key("S3_SECRET_ACCESS_KEY"));
    }

    #[test]
    fn test_minio_custom_port() {
        let config = ServiceConfig {
            port: Some(19000),
            ..default_config()
        };
        let (svc, _) = minio(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"19000:9000".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_minio_custom_console_port() {
        let mut extra = IndexMap::new();
        extra.insert("console_port".to_string(), toml::Value::Integer(19001));

        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, _) = minio(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"19001:9001".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_minio_custom_version() {
        let config = ServiceConfig {
            version: Some("RELEASE.2024-01-01".to_string()),
            ..default_config()
        };
        let (svc, _) = minio(&config).unwrap();
        assert_eq!(svc.image, Some("minio/minio:RELEASE.2024-01-01".to_string()));
    }

    #[test]
    fn test_minio_agent_env_has_three_keys() {
        let config = default_config();
        let (_, env) = minio(&config).unwrap();
        assert_eq!(env.len(), 3);
    }
}
