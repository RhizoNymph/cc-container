use crate::config::project::ServiceConfig;
use crate::error::Result;
use docker_compose_types as dct;
use indexmap::IndexMap;

fn get_str(config: &ServiceConfig, key: &str, default: &str) -> String {
    config
        .extra
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

pub fn postgres(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("16");
    let port = config.port.unwrap_or(5432);
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "POSTGRES_PASSWORD");

    let svc = dct::Service {
        image: Some(format!("postgres:{version}")),
        ports: dct::Ports::Short(vec![format!("{port}:5432")]),
        environment: dct::Environment::KvPair(IndexMap::from([
            ("POSTGRES_DB".to_string(), Some(dct::SingleValue::String(db.clone()))),
            ("POSTGRES_USER".to_string(), Some(dct::SingleValue::String(user.clone()))),
            ("POSTGRES_PASSWORD".to_string(), Some(dct::SingleValue::String(format!("${{{password_env}}}")))),
        ])),
        volumes: vec![dct::Volumes::Simple(format!("pgdata-{db}:/var/lib/postgresql/data"))],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "pg_isready -U ${POSTGRES_USER:-dev}".to_string(),
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
        "DATABASE_URL".to_string(),
        format!("postgres://{user}:${{{password_env}}}@postgres:{port}/{db}"),
    )]);

    Ok((svc, agent_env))
}

pub fn mysql(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("8");
    let port = config.port.unwrap_or(3306);
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "MYSQL_PASSWORD");
    let root_password_env = get_str(config, "root_password_env", "MYSQL_ROOT_PASSWORD");

    let svc = dct::Service {
        image: Some(format!("mysql:{version}")),
        ports: dct::Ports::Short(vec![format!("{port}:3306")]),
        environment: dct::Environment::KvPair(IndexMap::from([
            ("MYSQL_DATABASE".to_string(), Some(dct::SingleValue::String(db.clone()))),
            ("MYSQL_USER".to_string(), Some(dct::SingleValue::String(user.clone()))),
            ("MYSQL_PASSWORD".to_string(), Some(dct::SingleValue::String(format!("${{{password_env}}}")))),
            ("MYSQL_ROOT_PASSWORD".to_string(), Some(dct::SingleValue::String(format!("${{{root_password_env}}}")))),
        ])),
        volumes: vec![dct::Volumes::Simple(format!("mysqldata-{db}:/var/lib/mysql"))],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "mysqladmin ping -h localhost".to_string(),
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
        "DATABASE_URL".to_string(),
        format!("mysql://{user}:${{{password_env}}}@mysql:{port}/{db}"),
    )]);

    Ok((svc, agent_env))
}

pub fn mariadb(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("11");
    let port = config.port.unwrap_or(3306);
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "MARIADB_PASSWORD");
    let root_password_env = get_str(config, "root_password_env", "MARIADB_ROOT_PASSWORD");

    let svc = dct::Service {
        image: Some(format!("mariadb:{version}")),
        ports: dct::Ports::Short(vec![format!("{port}:3306")]),
        environment: dct::Environment::KvPair(IndexMap::from([
            ("MARIADB_DATABASE".to_string(), Some(dct::SingleValue::String(db.clone()))),
            ("MARIADB_USER".to_string(), Some(dct::SingleValue::String(user.clone()))),
            ("MARIADB_PASSWORD".to_string(), Some(dct::SingleValue::String(format!("${{{password_env}}}")))),
            ("MARIADB_ROOT_PASSWORD".to_string(), Some(dct::SingleValue::String(format!("${{{root_password_env}}}")))),
        ])),
        volumes: vec![dct::Volumes::Simple(format!("mariadbdata-{db}:/var/lib/mysql"))],
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "DATABASE_URL".to_string(),
        format!("mysql://{user}:${{{password_env}}}@mariadb:{port}/{db}"),
    )]);

    Ok((svc, agent_env))
}

pub fn mongodb(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("7");
    let port = config.port.unwrap_or(27017);

    let svc = dct::Service {
        image: Some(format!("mongo:{version}")),
        ports: dct::Ports::Short(vec![format!("{port}:27017")]),
        volumes: vec![dct::Volumes::Simple("mongodata:/data/db".to_string())],
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "MONGODB_URL".to_string(),
        format!("mongodb://mongodb:{port}"),
    )]);

    Ok((svc, agent_env))
}

pub fn cockroachdb(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let port = config.port.unwrap_or(26257);

    let svc = dct::Service {
        image: Some(format!("cockroachdb/cockroach:{version}")),
        command: Some(dct::Command::Simple("start-single-node --insecure".to_string())),
        ports: dct::Ports::Short(vec![
            format!("{port}:26257"),
            "8080:8080".to_string(),
        ]),
        volumes: vec![dct::Volumes::Simple("crdbdata:/cockroach/cockroach-data".to_string())],
        restart: Some("unless-stopped".to_string()),
        ..Default::default()
    };

    let agent_env = IndexMap::from([(
        "DATABASE_URL".to_string(),
        format!("postgres://root@cockroachdb:{port}/defaultdb?sslmode=disable"),
    )]);

    Ok((svc, agent_env))
}
