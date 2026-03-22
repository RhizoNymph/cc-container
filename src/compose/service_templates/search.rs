use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn elasticsearch(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("8");
    let port = config.port.unwrap_or(9200);

    let svc = dct::Service {
        image: Some(format!(
            "docker.elastic.co/elasticsearch/elasticsearch:{version}"
        )),
        ports: dct::Ports::Short(vec![format!("{port}:9200")]),
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
        format!("http://elasticsearch:{port}"),
    )]);

    Ok((svc, agent_env))
}

pub fn meilisearch(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(7700);

    let svc = dct::Service {
        image: Some(format!(
            "getmeili/meilisearch:{version}"
        )),
        ports: dct::Ports::Short(vec![format!("{port}:7700")]),
        volumes: vec![dct::Volumes::Simple("meilidata:/meili_data".to_string())],
        environment: dct::Environment::KvPair(IndexMap::from([
            ("MEILI_ENV".to_string(), Some(dct::SingleValue::String("development".to_string()))),
        ])),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "MEILISEARCH_URL".to_string(),
        format!("http://meilisearch:{port}"),
    )]);

    Ok((svc, agent_env))
}

pub fn typesense(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("27.1");
    let port = config.port.unwrap_or(8108);

    let svc = dct::Service {
        image: Some(format!("typesense/typesense:{version}")),
        ports: dct::Ports::Short(vec![format!("{port}:8108")]),
        volumes: vec![dct::Volumes::Simple("typesensedata:/data".to_string())],
        environment: dct::Environment::KvPair(IndexMap::from([
            ("TYPESENSE_API_KEY".to_string(), Some(dct::SingleValue::String("${TYPESENSE_API_KEY:-changeme}".to_string()))),
            ("TYPESENSE_DATA_DIR".to_string(), Some(dct::SingleValue::String("/data".to_string()))),
        ])),
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "TYPESENSE_URL".to_string(),
        format!("http://typesense:{port}"),
    )]);

    Ok((svc, agent_env))
}
