use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

pub fn minio(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(9000);

    let console_port = config
        .extra
        .get("console_port")
        .and_then(|v| v.as_integer())
        .unwrap_or(9001) as u16;

    let svc = dct::Service {
        image: Some(format!("minio/minio:{version}")),
        command: Some(dct::Command::Simple(format!(
            "server /data --console-address :{console_port}"
        ))),
        ports: dct::Ports::Short(vec![
            format!("{port}:9000"),
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
        ("S3_ENDPOINT".to_string(), format!("http://minio:{port}")),
        ("AWS_ACCESS_KEY_ID".to_string(), "${MINIO_ACCESS_KEY:-minioadmin}".to_string()),
        ("AWS_SECRET_ACCESS_KEY".to_string(), "${MINIO_SECRET_KEY:-minioadmin}".to_string()),
    ]);

    Ok((svc, agent_env))
}
