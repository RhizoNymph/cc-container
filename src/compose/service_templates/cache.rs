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
