use super::project::{AgentType, ProjectConfig, ServiceConfig};

/// A validation warning (non-fatal).
#[derive(Debug)]
pub struct ValidationWarning {
    pub message: String,
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn is_valid_project_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

fn is_valid_username(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c == '_')
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Validate a project config and return any warnings or errors.
pub fn validate_config(config: &ProjectConfig) -> crate::error::Result<Vec<ValidationWarning>> {
    let mut warnings = Vec::new();

    if !is_valid_project_name(&config.project.name) {
        return Err(crate::error::Error::Other(format!(
            "project.name '{}' is invalid: must start with alphanumeric and contain only [a-zA-Z0-9._-]",
            config.project.name
        )));
    }

    if !is_valid_username(&config.image.user) {
        return Err(crate::error::Error::Other(format!(
            "image.user '{}' is invalid: must be a valid Unix username matching [a-z_][a-z0-9_-]{{0,31}}",
            config.image.user
        )));
    }

    // Check that auth is configured for the selected agent type
    match config.agent.agent_type {
        AgentType::Claude | AgentType::Both => {
            if config.auth.claude.is_none() {
                warnings.push(ValidationWarning {
                    message: "No [auth.claude] configured. You will need to set up authentication."
                        .to_string(),
                });
            }
        }
        AgentType::Codex => {}
    }

    match config.agent.agent_type {
        AgentType::Codex | AgentType::Both => {
            if config.auth.codex.is_none() {
                warnings.push(ValidationWarning {
                    message: "No [auth.codex] configured. You will need to set up authentication."
                        .to_string(),
                });
            }
        }
        AgentType::Claude => {}
    }

    // Check for port conflicts among enabled services (primary + secondary ports)
    let mut ports: Vec<(u16, String)> = Vec::new();
    for (name, svc) in &config.services {
        if !svc.enabled {
            continue;
        }
        for (port, label) in all_ports_for_service(name, svc)? {
            for (existing_port, existing_label) in &ports {
                if *existing_port == port {
                    return Err(crate::error::Error::PortConflict {
                        port,
                        a: existing_label.clone(),
                        b: label.clone(),
                    });
                }
            }
            ports.push((port, label));
        }
    }

    // Warn if firewall is enabled but NET_ADMIN not in cap_add
    if config.firewall.enabled && !config.runtime.cap_add.iter().any(|c| c == "NET_ADMIN") {
        warnings.push(ValidationWarning {
            message:
                "Firewall is enabled but NET_ADMIN capability not in [runtime].cap_add. Firewall rules require NET_ADMIN."
                    .to_string(),
        });
    }

    Ok(warnings)
}

/// Parse a port number from `config.extra[key]`, returning `default` if the key
/// is absent and an error if the value is outside the valid 1–65535 range or
/// if the value is not an integer.
fn parse_extra_port(config: &ServiceConfig, key: &str, default: u16) -> crate::error::Result<u16> {
    match config.extra.get(key) {
        None => Ok(default),
        Some(v) => match v.as_integer() {
            Some(i) if (1..=65535).contains(&i) => Ok(i as u16),
            Some(i) => Err(crate::error::Error::InvalidPort {
                value: i,
                context: format!("extra.{key}"),
            }),
            None => Err(crate::error::Error::Other(format!(
                "{key} must be an integer, got: {v}"
            ))),
        },
    }
}

/// Returns all host-bound ports for a service: the primary configurable port
/// plus any configurable secondary ports emitted by the template.
fn all_ports_for_service(
    name: &str,
    config: &ServiceConfig,
) -> crate::error::Result<Vec<(u16, String)>> {
    let mut ports = Vec::new();

    let port = config.port.or_else(|| default_port_for_service(name));
    if let Some(port) = port {
        ports.push((port, name.to_string()));
    }

    // Secondary ports from service templates (configurable via extra)
    match name {
        "cockroachdb" => ports.push((8080, format!("{name} admin UI"))),
        "rabbitmq" => {
            let mgmt_port = parse_extra_port(config, "management_port", 15672)?;
            ports.push((mgmt_port, format!("{name} management")));
        }
        "kafka" => {
            let registry_port = parse_extra_port(config, "schema_registry_port", 8081)?;
            ports.push((registry_port, format!("{name} schema registry")));
        }
        "nats" => {
            let monitoring_port = parse_extra_port(config, "monitoring_port", 8222)?;
            ports.push((monitoring_port, format!("{name} monitoring")));
        }
        "traefik" => ports.push((8080, format!("{name} dashboard"))),
        "minio" => {
            let console_port = parse_extra_port(config, "console_port", 9001)?;
            ports.push((console_port, format!("{name} console")));
        }
        _ => {}
    }

    Ok(ports)
}

/// Returns the default port for a well-known service, matching the hardcoded
/// defaults used in the corresponding Docker Compose templates.
fn default_port_for_service(name: &str) -> Option<u16> {
    match name {
        // Databases
        "postgres" => Some(5432),
        "mysql" => Some(3306),
        "mariadb" => Some(3306),
        "mongodb" => Some(27017),
        "cockroachdb" => Some(26257),
        // Caches
        "redis" => Some(6379),
        "memcached" => Some(11211),
        // Message brokers
        "rabbitmq" => Some(5672),
        "kafka" => Some(9092),
        "nats" => Some(4222),
        // Search engines
        "elasticsearch" => Some(9200),
        "meilisearch" => Some(7700),
        "typesense" => Some(8108),
        // Infrastructure
        "minio" => Some(9000),
        "prometheus" => Some(9090),
        "grafana" => Some(3000),
        // Reverse proxies
        "traefik" => Some(80),
        "nginx" => Some(80),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::ProjectConfig;

    const MINIMAL_CLAUDE: &str = r#"
[project]
name = "test"
[agent]
type = "claude"
"#;

    const MINIMAL_CODEX: &str = r#"
[project]
name = "test"
[agent]
type = "codex"
"#;

    const MINIMAL_BOTH: &str = r#"
[project]
name = "test"
[agent]
type = "both"
"#;

    fn parse(toml_str: &str) -> ProjectConfig {
        toml::from_str(toml_str).unwrap()
    }

    // ───────────────────── Auth warnings ─────────────────────

    #[test]
    fn warn_missing_claude_auth_for_claude_agent() {
        let config = parse(MINIMAL_CLAUDE);
        let warnings = validate_config(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("[auth.claude]")));
    }

    #[test]
    fn no_codex_auth_warning_for_claude_agent() {
        let config = parse(MINIMAL_CLAUDE);
        let warnings = validate_config(&config).unwrap();
        assert!(!warnings.iter().any(|w| w.message.contains("[auth.codex]")));
    }

    #[test]
    fn warn_missing_codex_auth_for_codex_agent() {
        let config = parse(MINIMAL_CODEX);
        let warnings = validate_config(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("[auth.codex]")));
    }

    #[test]
    fn no_claude_auth_warning_for_codex_agent() {
        let config = parse(MINIMAL_CODEX);
        let warnings = validate_config(&config).unwrap();
        assert!(!warnings.iter().any(|w| w.message.contains("[auth.claude]")));
    }

    #[test]
    fn warn_missing_both_auth_for_both_agent() {
        let config = parse(MINIMAL_BOTH);
        let warnings = validate_config(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("[auth.claude]")));
        assert!(warnings.iter().any(|w| w.message.contains("[auth.codex]")));
    }

    #[test]
    fn no_auth_warning_when_configured() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "both"
[auth.claude]
method = "api-key"
[auth.codex]
method = "api-key"
"#;
        let config = parse(toml_str);
        let warnings = validate_config(&config).unwrap();
        assert!(!warnings.iter().any(|w| w.message.contains("[auth.claude]")));
        assert!(!warnings.iter().any(|w| w.message.contains("[auth.codex]")));
    }

    // ───────────────────── Port conflicts ─────────────────────

    #[test]
    fn no_port_conflict_for_different_ports() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.postgres]
port = 5432
[services.redis]
port = 6379
"#;
        let config = parse(toml_str);
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn detect_port_conflict_same_explicit_port() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.postgres]
port = 5432
[services.redis]
port = 5432
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::PortConflict { port, a, b } => {
                assert_eq!(port, 5432);
                assert!((a == "postgres" && b == "redis") || (a == "redis" && b == "postgres"));
            }
            other => panic!("Expected PortConflict, got: {:?}", other),
        }
    }

    #[test]
    fn detect_port_conflict_with_default_ports() {
        // mysql and mariadb both default to 3306
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.mysql]
enabled = true
[services.mariadb]
enabled = true
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::PortConflict { port, .. } => {
                assert_eq!(port, 3306);
            }
            other => panic!("Expected PortConflict, got: {:?}", other),
        }
    }

    #[test]
    fn disabled_services_no_port_conflict() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.postgres]
enabled = true
port = 5432
[services.redis]
enabled = false
port = 5432
"#;
        let config = parse(toml_str);
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn port_conflict_with_secondary_ports() {
        // cockroachdb has secondary port 8080 (admin UI)
        // traefik has secondary port 8080 (dashboard)
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.cockroachdb]
enabled = true
[services.traefik]
enabled = true
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::PortConflict { port, .. } => {
                assert_eq!(port, 8080);
            }
            other => panic!("Expected PortConflict for 8080, got: {:?}", other),
        }
    }

    #[test]
    fn no_port_conflict_with_no_services() {
        let config = parse(MINIMAL_CLAUDE);
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn no_port_conflict_single_service() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.postgres]
enabled = true
port = 5432
"#;
        let config = parse(toml_str);
        assert!(validate_config(&config).is_ok());
    }

    // ───────────────────── Firewall + NET_ADMIN ─────────────────────

    #[test]
    fn warn_firewall_without_net_admin() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[firewall]
enabled = true
"#;
        let config = parse(toml_str);
        let warnings = validate_config(&config).unwrap();
        assert!(warnings.iter().any(|w| w.message.contains("NET_ADMIN")));
    }

    #[test]
    fn no_firewall_warning_when_net_admin_present() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[firewall]
enabled = true
[runtime]
cap_add = ["NET_ADMIN"]
"#;
        let config = parse(toml_str);
        let warnings = validate_config(&config).unwrap();
        assert!(!warnings.iter().any(|w| w.message.contains("NET_ADMIN")));
    }

    #[test]
    fn no_firewall_warning_when_firewall_disabled() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[firewall]
enabled = false
"#;
        let config = parse(toml_str);
        let warnings = validate_config(&config).unwrap();
        assert!(!warnings.iter().any(|w| w.message.contains("NET_ADMIN")));
    }

    // ───────────────────── default_port_for_service ─────────────────────

    #[test]
    fn known_service_default_ports() {
        assert_eq!(default_port_for_service("postgres"), Some(5432));
        assert_eq!(default_port_for_service("mysql"), Some(3306));
        assert_eq!(default_port_for_service("mariadb"), Some(3306));
        assert_eq!(default_port_for_service("mongodb"), Some(27017));
        assert_eq!(default_port_for_service("cockroachdb"), Some(26257));
        assert_eq!(default_port_for_service("redis"), Some(6379));
        assert_eq!(default_port_for_service("memcached"), Some(11211));
        assert_eq!(default_port_for_service("rabbitmq"), Some(5672));
        assert_eq!(default_port_for_service("kafka"), Some(9092));
        assert_eq!(default_port_for_service("nats"), Some(4222));
        assert_eq!(default_port_for_service("elasticsearch"), Some(9200));
        assert_eq!(default_port_for_service("meilisearch"), Some(7700));
        assert_eq!(default_port_for_service("typesense"), Some(8108));
        assert_eq!(default_port_for_service("minio"), Some(9000));
        assert_eq!(default_port_for_service("prometheus"), Some(9090));
        assert_eq!(default_port_for_service("grafana"), Some(3000));
        assert_eq!(default_port_for_service("traefik"), Some(80));
        assert_eq!(default_port_for_service("nginx"), Some(80));
    }

    #[test]
    fn unknown_service_no_default_port() {
        assert_eq!(default_port_for_service("custom-svc"), None);
        assert_eq!(default_port_for_service(""), None);
    }

    // ───────────────────── all_ports_for_service ─────────────────────

    #[test]
    fn cockroachdb_secondary_port() {
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: indexmap::IndexMap::new(),
        };
        let ports = all_ports_for_service("cockroachdb", &svc).unwrap();
        assert!(ports.iter().any(|(p, _)| *p == 26257));
        assert!(ports.iter().any(|(p, _)| *p == 8080));
    }

    #[test]
    fn rabbitmq_secondary_port() {
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: indexmap::IndexMap::new(),
        };
        let ports = all_ports_for_service("rabbitmq", &svc).unwrap();
        assert!(ports.iter().any(|(p, _)| *p == 5672));
        assert!(ports.iter().any(|(p, _)| *p == 15672));
    }

    #[test]
    fn kafka_secondary_port() {
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: indexmap::IndexMap::new(),
        };
        let ports = all_ports_for_service("kafka", &svc).unwrap();
        assert!(ports.iter().any(|(p, _)| *p == 9092));
        assert!(ports.iter().any(|(p, _)| *p == 8081));
    }

    #[test]
    fn nats_secondary_port() {
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: indexmap::IndexMap::new(),
        };
        let ports = all_ports_for_service("nats", &svc).unwrap();
        assert!(ports.iter().any(|(p, _)| *p == 4222));
        assert!(ports.iter().any(|(p, _)| *p == 8222));
    }

    #[test]
    fn traefik_secondary_port() {
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: indexmap::IndexMap::new(),
        };
        let ports = all_ports_for_service("traefik", &svc).unwrap();
        assert!(ports.iter().any(|(p, _)| *p == 80));
        assert!(ports.iter().any(|(p, _)| *p == 8080));
    }

    #[test]
    fn minio_custom_console_port() {
        let mut extra = indexmap::IndexMap::new();
        extra.insert("console_port".to_string(), toml::Value::Integer(9002));
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra,
        };
        let ports = all_ports_for_service("minio", &svc).unwrap();
        assert!(ports.iter().any(|(p, _)| *p == 9000));
        assert!(ports.iter().any(|(p, _)| *p == 9002));
    }

    #[test]
    fn minio_default_console_port() {
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: indexmap::IndexMap::new(),
        };
        let ports = all_ports_for_service("minio", &svc).unwrap();
        assert!(ports.iter().any(|(p, _)| *p == 9000));
        assert!(ports.iter().any(|(p, _)| *p == 9001));
    }

    #[test]
    fn custom_port_overrides_default() {
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: Some(15432),
            extra: indexmap::IndexMap::new(),
        };
        let ports = all_ports_for_service("postgres", &svc).unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].0, 15432);
    }

    #[test]
    fn unknown_service_no_ports_without_explicit() {
        let svc = ServiceConfig {
            enabled: true,
            version: None,
            port: None,
            extra: indexmap::IndexMap::new(),
        };
        let ports = all_ports_for_service("unknown-service", &svc).unwrap();
        assert!(ports.is_empty());
    }

    // ───────────────────── ValidationWarning Display ─────────────────────

    // ───────────────────── Port range validation ─────────────────────

    #[test]
    fn test_invalid_port_over_65535() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.minio]
console_port = 70000
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::InvalidPort { value, context } => {
                assert_eq!(value, 70000);
                assert!(context.contains("console_port"));
            }
            other => panic!("Expected InvalidPort, got: {:?}", other),
        }
    }

    #[test]
    fn test_invalid_port_zero() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.minio]
console_port = 0
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::InvalidPort { value, .. } => {
                assert_eq!(value, 0);
            }
            other => panic!("Expected InvalidPort, got: {:?}", other),
        }
    }

    #[test]
    fn test_invalid_port_negative() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.minio]
console_port = -1
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::InvalidPort { value, .. } => {
                assert_eq!(value, -1);
            }
            other => panic!("Expected InvalidPort, got: {:?}", other),
        }
    }

    #[test]
    fn test_valid_custom_port_works() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.minio]
console_port = 9002
"#;
        let config = parse(toml_str);
        assert!(validate_config(&config).is_ok());
    }

    // ───────────────────── Configurable secondary ports in validation ─────────────────────

    #[test]
    fn test_rabbitmq_custom_management_port_validation() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.rabbitmq]
management_port = 25672
"#;
        let config = parse(toml_str);
        let warnings = validate_config(&config).unwrap();
        // Should succeed; just verify no port conflict errors
        let _ = warnings;
    }

    #[test]
    fn test_kafka_custom_schema_registry_port_validation() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.kafka]
schema_registry_port = 18081
"#;
        let config = parse(toml_str);
        let warnings = validate_config(&config).unwrap();
        let _ = warnings;
    }

    #[test]
    fn test_nats_custom_monitoring_port_validation() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.nats]
monitoring_port = 18222
"#;
        let config = parse(toml_str);
        let warnings = validate_config(&config).unwrap();
        let _ = warnings;
    }

    #[test]
    fn test_rabbitmq_invalid_management_port() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.rabbitmq]
management_port = 99999
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::InvalidPort { value, context } => {
                assert_eq!(value, 99999);
                assert!(context.contains("management_port"));
            }
            other => panic!("Expected InvalidPort, got: {:?}", other),
        }
    }

    #[test]
    fn test_kafka_invalid_schema_registry_port() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.kafka]
schema_registry_port = 99999
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::InvalidPort { value, context } => {
                assert_eq!(value, 99999);
                assert!(context.contains("schema_registry_port"));
            }
            other => panic!("Expected InvalidPort, got: {:?}", other),
        }
    }

    #[test]
    fn test_nats_invalid_monitoring_port() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.nats]
monitoring_port = 99999
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::InvalidPort { value, context } => {
                assert_eq!(value, 99999);
                assert!(context.contains("monitoring_port"));
            }
            other => panic!("Expected InvalidPort, got: {:?}", other),
        }
    }

    #[test]
    fn validation_warning_display() {
        let w = ValidationWarning {
            message: "test warning".to_string(),
        };
        assert_eq!(format!("{}", w), "test warning");
    }

    // ───────────────────── Full valid config, no warnings ─────────────────────

    #[test]
    fn fully_configured_has_no_warnings() {
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "both"
[auth.claude]
method = "api-key"
[auth.codex]
method = "api-key"
[firewall]
enabled = true
[runtime]
cap_add = ["NET_ADMIN"]
"#;
        let config = parse(toml_str);
        let warnings = validate_config(&config).unwrap();
        assert!(
            warnings.is_empty(),
            "Expected no warnings, got: {:?}",
            warnings
        );
    }

    // ───────────────────── Port conflict between primary and secondary ─────────────────────

    #[test]
    fn port_conflict_primary_vs_secondary() {
        // Set grafana to 8080 and add cockroachdb (secondary 8080)
        let toml_str = r#"
[project]
name = "test"
[agent]
type = "claude"
[services.grafana]
port = 8080
[services.cockroachdb]
enabled = true
"#;
        let config = parse(toml_str);
        let err = validate_config(&config).unwrap_err();
        match err {
            crate::error::Error::PortConflict { port, .. } => {
                assert_eq!(port, 8080);
            }
            other => panic!("Expected PortConflict, got: {:?}", other),
        }
    }
}
