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
    let host_port = config.port.unwrap_or(5432);
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "POSTGRES_PASSWORD");

    let svc = dct::Service {
        image: Some(format!("postgres:{version}")),
        ports: dct::Ports::Short(vec![format!("{host_port}:5432")]),
        environment: dct::Environment::KvPair(IndexMap::from([
            (
                "POSTGRES_DB".to_string(),
                Some(dct::SingleValue::String(db.clone())),
            ),
            (
                "POSTGRES_USER".to_string(),
                Some(dct::SingleValue::String(user.clone())),
            ),
            (
                "POSTGRES_PASSWORD".to_string(),
                Some(dct::SingleValue::String(format!("${{{password_env}}}"))),
            ),
        ])),
        volumes: vec![dct::Volumes::Simple(format!(
            "pgdata-{db}:/var/lib/postgresql/data"
        ))],
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
        format!("postgres://{user}:${{{password_env}}}@postgres:5432/{db}"),
    )]);

    Ok((svc, agent_env))
}

pub fn mysql(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("8");
    let host_port = config.port.unwrap_or(3306);
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "MYSQL_PASSWORD");
    let root_password_env = get_str(config, "root_password_env", "MYSQL_ROOT_PASSWORD");

    let svc = dct::Service {
        image: Some(format!("mysql:{version}")),
        ports: dct::Ports::Short(vec![format!("{host_port}:3306")]),
        environment: dct::Environment::KvPair(IndexMap::from([
            (
                "MYSQL_DATABASE".to_string(),
                Some(dct::SingleValue::String(db.clone())),
            ),
            (
                "MYSQL_USER".to_string(),
                Some(dct::SingleValue::String(user.clone())),
            ),
            (
                "MYSQL_PASSWORD".to_string(),
                Some(dct::SingleValue::String(format!("${{{password_env}}}"))),
            ),
            (
                "MYSQL_ROOT_PASSWORD".to_string(),
                Some(dct::SingleValue::String(format!(
                    "${{{root_password_env}}}"
                ))),
            ),
        ])),
        volumes: vec![dct::Volumes::Simple(format!(
            "mysqldata-{db}:/var/lib/mysql"
        ))],
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
        format!("mysql://{user}:${{{password_env}}}@mysql:3306/{db}"),
    )]);

    Ok((svc, agent_env))
}

pub fn mariadb(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("11");
    let host_port = config.port.unwrap_or(3306);
    let db = get_str(config, "database", "devdb");
    let user = get_str(config, "user", "dev");
    let password_env = get_str(config, "password_env", "MARIADB_PASSWORD");
    let root_password_env = get_str(config, "root_password_env", "MARIADB_ROOT_PASSWORD");

    let svc = dct::Service {
        image: Some(format!("mariadb:{version}")),
        ports: dct::Ports::Short(vec![format!("{host_port}:3306")]),
        environment: dct::Environment::KvPair(IndexMap::from([
            (
                "MARIADB_DATABASE".to_string(),
                Some(dct::SingleValue::String(db.clone())),
            ),
            (
                "MARIADB_USER".to_string(),
                Some(dct::SingleValue::String(user.clone())),
            ),
            (
                "MARIADB_PASSWORD".to_string(),
                Some(dct::SingleValue::String(format!("${{{password_env}}}"))),
            ),
            (
                "MARIADB_ROOT_PASSWORD".to_string(),
                Some(dct::SingleValue::String(format!(
                    "${{{root_password_env}}}"
                ))),
            ),
        ])),
        volumes: vec![dct::Volumes::Simple(format!(
            "mariadbdata-{db}:/var/lib/mysql"
        ))],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "mariadb-admin ping -h localhost".to_string(),
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
        format!("mysql://{user}:${{{password_env}}}@mariadb:3306/{db}"),
    )]);

    Ok((svc, agent_env))
}

pub fn mongodb(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("7");
    let host_port = config.port.unwrap_or(27017);

    let svc = dct::Service {
        image: Some(format!("mongo:{version}")),
        ports: dct::Ports::Short(vec![format!("{host_port}:27017")]),
        volumes: vec![dct::Volumes::Simple("mongodata:/data/db".to_string())],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "mongosh --eval 'db.runCommand(\"ping\").ok' --quiet".to_string(),
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
        "MONGODB_URL".to_string(),
        "mongodb://mongodb:27017".to_string(),
    )]);

    Ok((svc, agent_env))
}

pub fn cockroachdb(config: &ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)> {
    let version = config.version.as_deref().unwrap_or("latest");
    let host_port = config.port.unwrap_or(26257);

    let svc = dct::Service {
        image: Some(format!("cockroachdb/cockroach:{version}")),
        command: Some(dct::Command::Simple(
            "start-single-node --insecure".to_string(),
        )),
        ports: dct::Ports::Short(vec![format!("{host_port}:26257"), "8080:8080".to_string()]),
        volumes: vec![dct::Volumes::Simple(
            "crdbdata:/cockroach/cockroach-data".to_string(),
        )],
        healthcheck: Some(dct::Healthcheck {
            test: Some(dct::HealthcheckTest::Single(
                "curl -f http://localhost:8080/health?ready=1 || exit 1".to_string(),
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
        "postgres://root@cockroachdb:26257/defaultdb?sslmode=disable".to_string(),
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

    // --- Postgres tests ---

    #[test]
    fn test_postgres_defaults() {
        let config = default_config();
        let (svc, env) = postgres(&config).unwrap();

        assert_eq!(svc.image, Some("postgres:16".to_string()));
        assert_eq!(svc.restart, Some("unless-stopped".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["5432:5432"]),
            _ => panic!("Expected short ports"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        assert!(hc.test.is_some());
        assert_eq!(hc.retries, 5);

        assert_eq!(svc.volumes.len(), 1);
        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "pgdata-devdb:/var/lib/postgresql/data"),
            _ => panic!("Expected simple volume"),
        }

        if let dct::Environment::KvPair(e) = &svc.environment {
            assert!(e.contains_key("POSTGRES_DB"));
            assert!(e.contains_key("POSTGRES_USER"));
            assert!(e.contains_key("POSTGRES_PASSWORD"));
        } else {
            panic!("Expected KvPair environment");
        }

        assert!(env.contains_key("DATABASE_URL"));
        let url = &env["DATABASE_URL"];
        assert!(url.starts_with("postgres://dev:"));
        assert!(url.contains("@postgres:5432/devdb"));
    }

    #[test]
    fn test_postgres_custom_version() {
        let config = ServiceConfig {
            version: Some("15".to_string()),
            ..default_config()
        };
        let (svc, _) = postgres(&config).unwrap();
        assert_eq!(svc.image, Some("postgres:15".to_string()));
    }

    #[test]
    fn test_postgres_custom_port() {
        let config = ServiceConfig {
            port: Some(15432),
            ..default_config()
        };
        let (svc, _) = postgres(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["15432:5432"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_postgres_custom_database_and_user() {
        let mut extra = IndexMap::new();
        extra.insert(
            "database".to_string(),
            toml::Value::String("mydb".to_string()),
        );
        extra.insert(
            "user".to_string(),
            toml::Value::String("myuser".to_string()),
        );

        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, env) = postgres(&config).unwrap();

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "pgdata-mydb:/var/lib/postgresql/data"),
            _ => panic!("Expected simple volume"),
        }

        let url = &env["DATABASE_URL"];
        assert!(url.contains("myuser:"));
        assert!(url.contains("/mydb"));
    }

    #[test]
    fn test_postgres_custom_password_env() {
        let mut extra = IndexMap::new();
        extra.insert(
            "password_env".to_string(),
            toml::Value::String("MY_PG_PASS".to_string()),
        );

        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, env) = postgres(&config).unwrap();

        if let dct::Environment::KvPair(e) = &svc.environment {
            let pw_val = e.get("POSTGRES_PASSWORD").unwrap().as_ref().unwrap();
            match pw_val {
                dct::SingleValue::String(s) => assert_eq!(s, "${MY_PG_PASS}"),
                _ => panic!("Expected string value"),
            }
        }

        assert!(env["DATABASE_URL"].contains("${MY_PG_PASS}"));
    }

    // --- MySQL tests ---

    #[test]
    fn test_mysql_defaults() {
        let config = default_config();
        let (svc, env) = mysql(&config).unwrap();

        assert_eq!(svc.image, Some("mysql:8".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["3306:3306"]),
            _ => panic!("Expected short ports"),
        }

        if let dct::Environment::KvPair(e) = &svc.environment {
            assert!(e.contains_key("MYSQL_DATABASE"));
            assert!(e.contains_key("MYSQL_USER"));
            assert!(e.contains_key("MYSQL_PASSWORD"));
            assert!(e.contains_key("MYSQL_ROOT_PASSWORD"));
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "mysqldata-devdb:/var/lib/mysql"),
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("mysqladmin ping"));
        }

        assert!(env["DATABASE_URL"].starts_with("mysql://dev:"));
    }

    #[test]
    fn test_mysql_custom_port() {
        let config = ServiceConfig {
            port: Some(13306),
            ..default_config()
        };
        let (svc, _) = mysql(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["13306:3306"]),
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_mysql_custom_password_envs() {
        let mut extra = IndexMap::new();
        extra.insert(
            "password_env".to_string(),
            toml::Value::String("CUSTOM_PW".to_string()),
        );
        extra.insert(
            "root_password_env".to_string(),
            toml::Value::String("CUSTOM_ROOT_PW".to_string()),
        );

        let config = ServiceConfig {
            extra,
            ..default_config()
        };
        let (svc, _) = mysql(&config).unwrap();

        if let dct::Environment::KvPair(e) = &svc.environment {
            let pw = e.get("MYSQL_PASSWORD").unwrap().as_ref().unwrap();
            match pw {
                dct::SingleValue::String(s) => assert_eq!(s, "${CUSTOM_PW}"),
                _ => panic!("Expected string"),
            }
            let root_pw = e.get("MYSQL_ROOT_PASSWORD").unwrap().as_ref().unwrap();
            match root_pw {
                dct::SingleValue::String(s) => assert_eq!(s, "${CUSTOM_ROOT_PW}"),
                _ => panic!("Expected string"),
            }
        }
    }

    // --- MariaDB tests ---

    #[test]
    fn test_mariadb_defaults() {
        let config = default_config();
        let (svc, env) = mariadb(&config).unwrap();

        assert_eq!(svc.image, Some("mariadb:11".to_string()));

        if let dct::Environment::KvPair(e) = &svc.environment {
            assert!(e.contains_key("MARIADB_DATABASE"));
            assert!(e.contains_key("MARIADB_USER"));
            assert!(e.contains_key("MARIADB_PASSWORD"));
            assert!(e.contains_key("MARIADB_ROOT_PASSWORD"));
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "mariadbdata-devdb:/var/lib/mysql"),
            _ => panic!("Expected simple volume"),
        }

        let hc = svc.healthcheck.as_ref().unwrap();
        if let Some(dct::HealthcheckTest::Single(cmd)) = &hc.test {
            assert!(cmd.contains("mariadb-admin ping"));
        }

        assert!(env["DATABASE_URL"].starts_with("mysql://dev:"));
        assert!(env["DATABASE_URL"].contains("@mariadb:3306/devdb"));
    }

    #[test]
    fn test_mariadb_custom_port() {
        let config = ServiceConfig {
            port: Some(23306),
            ..default_config()
        };
        let (svc, _) = mariadb(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["23306:3306"]),
            _ => panic!("Expected short ports"),
        }
    }

    // --- MongoDB tests ---

    #[test]
    fn test_mongodb_defaults() {
        let config = default_config();
        let (svc, env) = mongodb(&config).unwrap();

        assert_eq!(svc.image, Some("mongo:7".to_string()));

        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["27017:27017"]),
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "mongodata:/data/db"),
            _ => panic!("Expected simple volume"),
        }

        assert_eq!(env["MONGODB_URL"], "mongodb://mongodb:27017");
    }

    #[test]
    fn test_mongodb_custom_version() {
        let config = ServiceConfig {
            version: Some("6".to_string()),
            ..default_config()
        };
        let (svc, _) = mongodb(&config).unwrap();
        assert_eq!(svc.image, Some("mongo:6".to_string()));
    }

    #[test]
    fn test_mongodb_custom_port() {
        let config = ServiceConfig {
            port: Some(37017),
            ..default_config()
        };
        let (svc, _) = mongodb(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => assert_eq!(ports, &["37017:27017"]),
            _ => panic!("Expected short ports"),
        }
    }

    // --- CockroachDB tests ---

    #[test]
    fn test_cockroachdb_defaults() {
        let config = default_config();
        let (svc, env) = cockroachdb(&config).unwrap();

        assert_eq!(svc.image, Some("cockroachdb/cockroach:latest".to_string()));

        match &svc.command {
            Some(dct::Command::Simple(cmd)) => assert!(cmd.contains("start-single-node")),
            _ => panic!("Expected simple command"),
        }

        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert_eq!(ports.len(), 2);
                assert!(ports.contains(&"26257:26257".to_string()));
                assert!(ports.contains(&"8080:8080".to_string()));
            }
            _ => panic!("Expected short ports"),
        }

        match &svc.volumes[0] {
            dct::Volumes::Simple(v) => assert_eq!(v, "crdbdata:/cockroach/cockroach-data"),
            _ => panic!("Expected simple volume"),
        }

        assert!(env["DATABASE_URL"].contains("cockroachdb:26257"));
    }

    #[test]
    fn test_cockroachdb_custom_port() {
        let config = ServiceConfig {
            port: Some(36257),
            ..default_config()
        };
        let (svc, _) = cockroachdb(&config).unwrap();
        match &svc.ports {
            dct::Ports::Short(ports) => {
                assert!(ports.contains(&"36257:26257".to_string()));
                assert!(ports.contains(&"8080:8080".to_string()));
            }
            _ => panic!("Expected short ports"),
        }
    }

    #[test]
    fn test_all_databases_have_healthchecks() {
        let config = default_config();

        for (name, builder) in [
            (
                "postgres",
                postgres as fn(&ServiceConfig) -> Result<(dct::Service, IndexMap<String, String>)>,
            ),
            ("mysql", mysql),
            ("mariadb", mariadb),
            ("mongodb", mongodb),
            ("cockroachdb", cockroachdb),
        ] {
            let (svc, _) = builder(&config).unwrap();
            assert!(
                svc.healthcheck.is_some(),
                "{} should have a healthcheck",
                name
            );
            let hc = svc.healthcheck.as_ref().unwrap();
            assert!(hc.test.is_some(), "{} healthcheck should have a test", name);
            assert_eq!(hc.retries, 5, "{} healthcheck retries should be 5", name);
        }
    }

    #[test]
    fn test_all_databases_have_restart_policy() {
        let config = default_config();

        for builder in [postgres, mysql, mariadb, mongodb, cockroachdb] {
            let (svc, _) = builder(&config).unwrap();
            assert_eq!(svc.restart, Some("unless-stopped".to_string()));
        }
    }
}
