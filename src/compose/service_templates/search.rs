use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn elasticsearch(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("8");
    let host_port = config.port.unwrap_or(9200);

    let svc = dct::Service {
        image: Some(format!(
            "docker.elastic.co/elasticsearch/elasticsearch:{version}"
        )),
        ports: dct::Ports::Short(vec![format!("{host_port}:9200")]),
        environment: dct::Environment::KvPair(IndexMap::from([
            ("discovery.type".to_string(), Some(dct::SingleValue::String("single-node".to_string()))),
            ("xpack.security.enabled".to_string(), Some(dct::SingleValue::String("false".to_string()))),
            ("ES_JAVA_OPTS".to_string(), Some(dct::SingleValue::String("-Xms512m -Xmx512m".to_string()))),
        ])),
        volumes: vec![dct::Volumes::Simple("esdata:/usr/share/elasticsearch/data".to_string())],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "curl -s http://localhost:9200/_cluster/health || exit 1".to_string(),
            )),
            interval: Some("15s".to_string()),
            timeout: Some("10s".to_string()),
            retries: 5,
            start_period: Some("30s".to_string()),
            ..Default::default()
        }),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "ELASTICSEARCH_URL".to_string(),
        "http://elasticsearch:9200".to_string(),
    )]);

    Ok((svc, agent_env))
}

pub fn meilisearch(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let host_port = config.port.unwrap_or(7700);

    let svc = dct::Service {
        image: Some(format!(
            "getmeili/meilisearch:{version}"
        )),
        ports: dct::Ports::Short(vec![format!("{host_port}:7700")]),
        volumes: vec![dct::Volumes::Simple("meilidata:/meili_data".to_string())],
        environment: dct::Environment::KvPair(IndexMap::from([
            ("MEILI_ENV".to_string(), Some(dct::SingleValue::String("development".to_string()))),
        ])),
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "curl -f http://localhost:7700/health || exit 1".to_string(),
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
        "MEILISEARCH_URL".to_string(),
        "http://meilisearch:7700".to_string(),
    )]);

    Ok((svc, agent_env))
}

pub fn typesense(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("27.1");
    let host_port = config.port.unwrap_or(8108);

    let svc = dct::Service {
        image: Some(format!("typesense/typesense:{version}")),
        ports: dct::Ports::Short(vec![format!("{host_port}:8108")]),
        volumes: vec![dct::Volumes::Simple("typesensedata:/data".to_string())],
        environment: dct::Environment::KvPair(IndexMap::from([
            ("TYPESENSE_API_KEY".to_string(), Some(dct::SingleValue::String("${TYPESENSE_API_KEY:-changeme}".to_string()))),
            ("TYPESENSE_DATA_DIR".to_string(), Some(dct::SingleValue::String("/data".to_string()))),
        ])),
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "curl -f http://localhost:8108/health || exit 1".to_string(),
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
        "TYPESENSE_URL".to_string(),
        "http://typesense:8108".to_string(),
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
    fn test_elasticsearch_defaults() {
        let config = default_config();
        let (svc, env) = elasticsearch(&config).unwrap();

        assert_eq!(
            svc.image,
            Some("docker.elastic.co/elasticsearch/elasticsearch:8".to_string())
        );

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["9200:9200"]),
            _ => panic!("Expected short ports"),
        }

        if let dct::Environment::KvPair(e) = &svc.environment {
            assert!(e.contains_key("discovery.type"));
            assert!(e.contains_key("xpack.security.enabled"));
            assert!(e.contains_key("ES_JAVA_OPTS"));
        } else {
            panic!("Expected KvPair environment");
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "esdata:/usr/share/elasticsearch/data"),
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        assert_eq!(hc.interval, Some("15s".to_string()));
        assert_eq!(hc.timeout, Some("10s".to_string()));

        assert_eq!(env["ELASTICSEARCH_URL"], "http://elasticsearch:9200");
    }

    #[test]
    fn test_elasticsearch_custom_version() {
        let config = ServiceConfig {
            version: Some("7.17".to_string()),
            ..default_config()
        };
        let (svc, _) = elasticsearch(&config).unwrap();
        assert_eq!(
            svc.image,
            Some("docker.elastic.co/elasticsearch/elasticsearch:7.17".to_string())
        );
    }

    #[test]
    fn test_elasticsearch_custom_port() {
        let config = ServiceConfig {
            port: Some(19200),
            ..default_config()
        };
        let (svc, _) = elasticsearch(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["19200:9200"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_meilisearch_defaults() {
        let config = default_config();
        let (svc, env) = meilisearch(&config).unwrap();

        assert_eq!(svc.image, Some("getmeili/meilisearch:latest".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["7700:7700"]),
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "meilidata:/meili_data"),
            _ => panic!("Expected simple volume"),
        }

        if let dct::Environment::KvPair(e) = &svc.environment {
            assert!(e.contains_key("MEILI_ENV"));
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("7700/health"));
        }

        assert_eq!(env["MEILISEARCH_URL"], "http://meilisearch:7700");
    }

    #[test]
    fn test_meilisearch_custom_port() {
        let config = ServiceConfig {
            port: Some(17700),
            ..default_config()
        };
        let (svc, _) = meilisearch(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["17700:7700"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_typesense_defaults() {
        let config = default_config();
        let (svc, env) = typesense(&config).unwrap();

        assert_eq!(svc.image, Some("typesense/typesense:27.1".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["8108:8108"]),
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "typesensedata:/data"),
            _ => panic!("Expected simple volume"),
        }

        if let dct::Environment::KvPair(e) = &svc.environment {
            assert!(e.contains_key("TYPESENSE_API_KEY"));
            assert!(e.contains_key("TYPESENSE_DATA_DIR"));
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("8108/health"));
        }

        assert_eq!(env["TYPESENSE_URL"], "http://typesense:8108");
    }

    #[test]
    fn test_typesense_custom_version() {
        let config = ServiceConfig {
            version: Some("26.0".to_string()),
            ..default_config()
        };
        let (svc, _) = typesense(&config).unwrap();
        assert_eq!(svc.image, Some("typesense/typesense:26.0".to_string()));
    }

    #[test]
    fn test_typesense_custom_port() {
        let config = ServiceConfig {
            port: Some(18108),
            ..default_config()
        };
        let (svc, _) = typesense(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["18108:8108"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_search_services_have_healthchecks() {
        let config = default_config();
        for builder in [elasticsearch, meilisearch, typesense] {
            let (svc, _) = builder(&config).unwrap();
            assert!(svc.healthcheck.is_some());
            let hc = svc.healthcheck.unwrap();
            assert_eq!(hc.retries, 5);
        }
    }

    #[test]
    fn test_search_services_have_restart_policy() {
        let config = default_config();
        for builder in [elasticsearch, meilisearch, typesense] {
            let (svc, _) = builder(&config).unwrap();
            assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        }
    }

    #[test]
    fn test_search_services_have_volumes() {
        let config = default_config();
        for builder in [elasticsearch, meilisearch, typesense] {
            let (svc, _) = builder(&config).unwrap();
            assert!(!svc.volumes.is_empty());
        }
    }
}
