use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn redis(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("7-alpine");
    let host_port = config.port.unwrap_or(6379);

    let svc = dct::Service {
        image: Some(format!("redis:{version}")),
        ports: dct::Ports::Short(vec![format!("{host_port}:6379")]),
        volumes: vec![dct::Volumes::Simple("redisdata:/data".to_string())],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "redis-cli ping".to_string(),
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
        "REDIS_URL".to_string(),
        "redis://redis:6379".to_string(),
    )]);

    Ok((svc, agent_env))
}

pub fn memcached(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("1-alpine");
    let host_port = config.port.unwrap_or(11211);

    let svc = dct::Service {
        image: Some(format!("memcached:{version}")),
        ports: dct::Ports::Short(vec![format!("{host_port}:11211")]),
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "echo stats | nc localhost 11211 || exit 1".to_string(),
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
        "MEMCACHED_URL".to_string(),
        "memcached:11211".to_string(),
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
    fn test_redis_defaults() {
        let config = default_config();
        let (svc, env) = redis(&config).unwrap();

        assert_eq!(svc.image, Some("redis:7-alpine".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["6379:6379"]),
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "redisdata:/data"),
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert_eq!(cmd, "redis-cli ping");
        } else {
            panic!("Expected single healthcheck test");
        }

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        assert_eq!(env["REDIS_URL"], "redis://redis:6379");
    }

    #[test]
    fn test_redis_custom_version() {
        let config = ServiceConfig {
            version: Some("6-alpine".to_string()),
            ..default_config()
        };
        let (svc, _) = redis(&config).unwrap();
        assert_eq!(svc.image, Some("redis:6-alpine".to_string()));
    }

    #[test]
    fn test_redis_custom_port() {
        let config = ServiceConfig {
            port: Some(16379),
            ..default_config()
        };
        let (svc, _) = redis(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["16379:6379"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_memcached_defaults() {
        let config = default_config();
        let (svc, env) = memcached(&config).unwrap();

        assert_eq!(svc.image, Some("memcached:1-alpine".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["11211:11211"]),
            _ => panic!("Expected short ports"),
        }

        assert!(svc.volumes.is_empty());

        let hc = svc.healthcheck.as_ref().unwrap();
        assert!(hc.test.is_some());
        assert_eq!(hc.retries, 5);

        assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        assert_eq!(env["MEMCACHED_URL"], "memcached:11211");
    }

    #[test]
    fn test_memcached_custom_port() {
        let config = ServiceConfig {
            port: Some(21211),
            ..default_config()
        };
        let (svc, _) = memcached(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["21211:11211"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_memcached_custom_version() {
        let config = ServiceConfig {
            version: Some("1.6".to_string()),
            ..default_config()
        };
        let (svc, _) = memcached(&config).unwrap();
        assert_eq!(svc.image, Some("memcached:1.6".to_string()));
    }

    #[test]
    fn test_cache_services_have_healthchecks() {
        let config = default_config();
        for builder in [redis, memcached] {
            let (svc, _) = builder(&config).unwrap();
            assert!(svc.healthcheck.is_some());
        }
    }
}
