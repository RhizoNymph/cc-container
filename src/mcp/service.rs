// MCP compose service generation is handled directly in compose/generator.rs
// This module exists for future expansion (e.g., MCP server discovery, validation).

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::compose::generator::generate;
    use crate::config::project::ProjectConfig;
    use docker_compose_types as dct;

    /// Helper: build a ProjectConfig from a TOML string.
    fn parse_config(toml_str: &str) -> ProjectConfig {
        toml::from_str(toml_str).expect("valid TOML for test config")
    }

    /// Minimal TOML that satisfies the required ProjectConfig fields.
    const BASE_TOML: &str = r#"
[project]
name = "test"

[agent]
type = "claude"
"#;

    /// Helper: extract a Service from a Compose by service name.
    fn get_service(compose: &dct::Compose, name: &str) -> dct::Service {
        compose
            .services
            .0
            .get(name)
            .unwrap_or_else(|| panic!("service '{name}' not found"))
            .clone()
            .unwrap_or_else(|| panic!("service '{name}' is None"))
    }

    /// Helper: check if a service exists in the compose output.
    fn has_service(compose: &dct::Compose, name: &str) -> bool {
        compose.services.0.contains_key(name)
    }

    // ── No MCP servers produces no mcp- services ────────────────────

    #[test]
    fn no_mcp_config_produces_no_mcp_services() {
        let config = parse_config(BASE_TOML);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        let mcp_services: Vec<_> = compose
            .services
            .0
            .keys()
            .filter(|k| k.starts_with("mcp-"))
            .collect();

        assert!(mcp_services.is_empty());
    }

    // ── Single MCP server with minimal config ───────────────────────

    #[test]
    fn single_mcp_server_creates_prefixed_service() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        assert!(has_service(&compose, "mcp-fetch"));
    }

    #[test]
    fn mcp_service_has_correct_image() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        assert_eq!(svc.image.unwrap(), "mcp/fetch:latest");
    }

    #[test]
    fn mcp_service_minimal_has_no_command() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        assert!(svc.command.is_none());
    }

    #[test]
    fn mcp_service_minimal_has_empty_ports() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        match svc.ports {
            dct::Ports::Short(ref v) => assert!(v.is_empty()),
            _ => panic!("expected Ports::Short"),
        }
    }

    #[test]
    fn mcp_service_minimal_has_empty_volumes() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        assert!(svc.volumes.is_empty());
    }

    #[test]
    fn mcp_service_minimal_has_empty_environment() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        match svc.environment {
            dct::Environment::KvPair(ref map) => assert!(map.is_empty()),
            _ => panic!("expected Environment::KvPair"),
        }
    }

    #[test]
    fn mcp_service_has_unless_stopped_restart() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        assert_eq!(svc.restart.as_deref(), Some("unless-stopped"));
    }

    // ── MCP server with command ─────────────────────────────────────

    #[test]
    fn mcp_service_with_command() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
command = ["node", "server.js"]
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        match svc.command {
            Some(dct::Command::Args(ref args)) => {
                assert_eq!(args, &vec!["node".to_string(), "server.js".to_string()]);
            }
            other => panic!("expected Command::Args, got {:?}", other),
        }
    }

    // ── MCP server with env vars ────────────────────────────────────

    #[test]
    fn mcp_service_with_env_vars() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
env = ["API_KEY", "DEBUG"]
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        match svc.environment {
            dct::Environment::KvPair(ref map) => {
                assert_eq!(map.len(), 2);
                assert!(map.contains_key("API_KEY"));
                assert!(map.contains_key("DEBUG"));
            }
            _ => panic!("expected Environment::KvPair"),
        }
    }

    // ── MCP server with volumes ─────────────────────────────────────

    #[test]
    fn mcp_service_with_volumes() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
volumes = ["/data:/data", "/config:/config:ro"]
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        assert_eq!(svc.volumes.len(), 2);

        let vol_strings: Vec<String> = svc
            .volumes
            .iter()
            .map(|v| match v {
                dct::Volumes::Simple(s) => s.clone(),
                _ => panic!("expected Volumes::Simple"),
            })
            .collect();

        assert!(vol_strings.contains(&"/data:/data".to_string()));
        assert!(vol_strings.contains(&"/config:/config:ro".to_string()));
    }

    // ── MCP server with port ────────────────────────────────────────

    #[test]
    fn mcp_service_with_port_mapping() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
port = 8000
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        match svc.ports {
            dct::Ports::Short(ref v) => {
                assert_eq!(v.len(), 1);
                assert_eq!(v[0], "8000:8000");
            }
            _ => panic!("expected Ports::Short"),
        }
    }

    // ── MCP server with all fields ──────────────────────────────────

    #[test]
    fn mcp_service_with_all_fields() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
command = ["node", "server.js"]
env = ["API_KEY"]
volumes = ["/data:/data"]
port = 8000
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        // Image
        assert_eq!(svc.image.as_deref(), Some("mcp/fetch:latest"));

        // Command
        match svc.command {
            Some(dct::Command::Args(ref args)) => {
                assert_eq!(args, &vec!["node".to_string(), "server.js".to_string()]);
            }
            other => panic!("expected Command::Args, got {:?}", other),
        }

        // Env
        match svc.environment {
            dct::Environment::KvPair(ref map) => {
                assert_eq!(map.len(), 1);
            }
            _ => panic!("expected Environment::KvPair"),
        }

        // Volumes
        assert_eq!(svc.volumes.len(), 1);

        // Ports
        match svc.ports {
            dct::Ports::Short(ref v) => {
                assert_eq!(v, &vec!["8000:8000"]);
            }
            _ => panic!("expected Ports::Short"),
        }

        // Restart policy
        assert_eq!(svc.restart.as_deref(), Some("unless-stopped"));
    }

    // ── Multiple MCP servers ────────────────────────────────────────

    #[test]
    fn multiple_mcp_servers_create_separate_services() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"

[mcp.memory]
image = "mcp/memory:latest"

[mcp.search]
image = "mcp/search:v2"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        assert!(has_service(&compose, "mcp-fetch"));
        assert!(has_service(&compose, "mcp-memory"));
        assert!(has_service(&compose, "mcp-search"));
    }

    #[test]
    fn multiple_mcp_servers_each_has_own_image() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"

[mcp.memory]
image = "mcp/memory:v2"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        let fetch = get_service(&compose, "mcp-fetch");
        let memory = get_service(&compose, "mcp-memory");

        assert_eq!(fetch.image.as_deref(), Some("mcp/fetch:latest"));
        assert_eq!(memory.image.as_deref(), Some("mcp/memory:v2"));
    }

    #[test]
    fn multiple_mcp_servers_with_different_ports() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
port = 8000

[mcp.memory]
image = "mcp/memory:latest"
port = 9000
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        let fetch = get_service(&compose, "mcp-fetch");
        let memory = get_service(&compose, "mcp-memory");

        match fetch.ports {
            dct::Ports::Short(ref v) => assert_eq!(v, &vec!["8000:8000"]),
            _ => panic!("expected Ports::Short for fetch"),
        }
        match memory.ports {
            dct::Ports::Short(ref v) => assert_eq!(v, &vec!["9000:9000"]),
            _ => panic!("expected Ports::Short for memory"),
        }
    }

    // ── MCP service naming convention ───────────────────────────────

    #[test]
    fn mcp_service_name_is_prefixed_with_mcp_dash() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.my-server]
image = "img:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        assert!(has_service(&compose, "mcp-my-server"));
    }

    // ── MCP services coexist with agent service ─────────────────────

    #[test]
    fn mcp_services_coexist_with_agent_service() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        // Both agent and MCP services should exist
        assert!(has_service(&compose, "agent"));
        assert!(has_service(&compose, "mcp-fetch"));
    }

    // ── MCP named volumes appear in top-level volumes ───────────────

    #[test]
    fn mcp_named_volumes_appear_in_top_level() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
volumes = ["mcp-data:/data"]
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        assert!(
            compose.volumes.0.contains_key("mcp-data"),
            "named volume 'mcp-data' should appear in top-level volumes"
        );
    }

    #[test]
    fn mcp_host_path_volumes_not_in_top_level() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
volumes = ["/host/path:/container/path"]
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();

        // Host-path volumes (starting with /) should not be in top-level volumes
        assert!(
            !compose.volumes.0.contains_key("/host/path"),
            "host path volumes should not appear in top-level volumes"
        );
    }

    // ── Env var values use ${} substitution ─────────────────────────

    #[test]
    fn mcp_env_vars_use_dollar_brace_substitution() {
        let toml_str = r#"
[project]
name = "test"

[agent]
type = "claude"

[mcp.fetch]
image = "mcp/fetch:latest"
env = ["MY_KEY"]
"#;
        let config = parse_config(toml_str);
        let compose = generate(&config, Path::new("."), Path::new(".")).unwrap();
        let svc = get_service(&compose, "mcp-fetch");

        match svc.environment {
            dct::Environment::KvPair(ref map) => {
                let val = map.get("MY_KEY").unwrap();
                match val {
                    Some(dct::SingleValue::String(s)) => {
                        assert_eq!(s, "${MY_KEY}");
                    }
                    other => panic!("expected SingleValue::String, got {:?}", other),
                }
            }
            _ => panic!("expected Environment::KvPair"),
        }
    }
}
