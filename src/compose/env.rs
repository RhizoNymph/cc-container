use crate::auth;
use crate::config::project::{AgentType, ProjectConfig};

/// Extract the KEY part from an env var string.
/// Handles "KEY", "KEY=value", and "KEY=${KEY}" formats.
pub fn parse_env_key(env_str: &str) -> &str {
    env_str.split_once('=').map(|(k, _)| k).unwrap_or(env_str)
}

/// Generate the contents of a .env.example file.
pub fn generate_env_example(config: &ProjectConfig) -> String {
    let mut lines = Vec::new();

    lines.push("# cc-container environment variables".to_string());
    lines.push("# Copy this file to .env and fill in your values.".to_string());
    lines.push(String::new());

    // Auth section
    let container_user = &config.image.user;

    match config.agent.agent_type {
        AgentType::Claude | AgentType::Both => {
            if let Some(ref claude_auth) = config.auth.claude {
                let reqs = auth::claude::requirements(claude_auth, container_user);
                for line in &reqs.env_example_lines {
                    lines.push(line.clone());
                }
                lines.push(String::new());
            }
        }
        _ => {}
    }

    match config.agent.agent_type {
        AgentType::Codex | AgentType::Both => {
            if let Some(ref codex_auth) = config.auth.codex {
                let reqs = auth::codex::requirements(codex_auth, container_user);
                for line in &reqs.env_example_lines {
                    lines.push(line.clone());
                }
                lines.push(String::new());
            }
        }
        _ => {}
    }

    // Service passwords and connection details
    let has_services = config.services.values().any(|s| s.enabled);
    if has_services {
        lines.push("# Service credentials".to_string());

        for (name, svc_config) in &config.services {
            if !svc_config.enabled {
                continue;
            }

            match name.as_str() {
                "postgres" => {
                    let pw_env = svc_config
                        .extra
                        .get("password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("POSTGRES_PASSWORD");
                    lines.push(format!("{pw_env}=changeme"));
                }
                "mysql" => {
                    let pw_env = svc_config
                        .extra
                        .get("password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MYSQL_PASSWORD");
                    let root_pw_env = svc_config
                        .extra
                        .get("root_password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MYSQL_ROOT_PASSWORD");
                    lines.push(format!("{pw_env}=changeme"));
                    lines.push(format!("{root_pw_env}=changeme"));
                }
                "mariadb" => {
                    let pw_env = svc_config
                        .extra
                        .get("password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MARIADB_PASSWORD");
                    let root_pw_env = svc_config
                        .extra
                        .get("root_password_env")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MARIADB_ROOT_PASSWORD");
                    lines.push(format!("{pw_env}=changeme"));
                    lines.push(format!("{root_pw_env}=changeme"));
                }
                "minio" => {
                    lines.push("MINIO_ACCESS_KEY=minioadmin".to_string());
                    lines.push("MINIO_SECRET_KEY=minioadmin".to_string());
                }
                _ => {}
            }
        }
        lines.push(String::new());
    }

    // MCP env vars
    if !config.mcp.is_empty() {
        lines.push("# MCP server credentials".to_string());
        for (_name, mcp_config) in &config.mcp {
            for env_var in &mcp_config.env {
                let key = parse_env_key(env_var);
                lines.push(format!("{key}=your-value-here"));
            }
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::project::*;
    use indexmap::IndexMap;

    fn minimal_config() -> ProjectConfig {
        ProjectConfig {
            project: ProjectMeta {
                name: "test-project".to_string(),
                description: None,
            },
            agent: AgentConfig {
                agent_type: AgentType::Claude,
                claude_version: "latest".to_string(),
                codex_version: "latest".to_string(),
            },
            image: ImageConfig::default(),
            modules: IndexMap::new(),
            auth: AuthConfig::default(),
            firewall: FirewallConfig::default(),
            workspace: WorkspaceConfig::default(),
            volumes: IndexMap::new(),
            environment: EnvironmentConfig::default(),
            services: IndexMap::new(),
            mcp: IndexMap::new(),
            runtime: RuntimeConfig::default(),
            helm: HelmConfig::default(),
        }
    }

    #[test]
    fn test_env_example_minimal() {
        let config = minimal_config();
        let output = generate_env_example(&config);

        assert!(output.contains("# cc-container environment variables"));
        assert!(output.contains("# Copy this file to .env"));
    }

    #[test]
    fn test_env_example_no_auth_no_services() {
        let config = minimal_config();
        let output = generate_env_example(&config);

        assert!(!output.contains("# Service credentials"));
        assert!(!output.contains("# MCP server credentials"));
    }

    #[test]
    fn test_env_example_claude_api_key() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::ApiKey,
        });

        let output = generate_env_example(&config);
        assert!(output.contains("ANTHROPIC_API_KEY="));
    }

    #[test]
    fn test_env_example_claude_oauth() {
        let mut config = minimal_config();
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::Oauth,
        });

        let output = generate_env_example(&config);
        assert!(output.contains("OAuth"));
    }

    #[test]
    fn test_env_example_codex_api_key() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Codex;
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::ApiKey,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });

        let output = generate_env_example(&config);
        assert!(output.contains("OPENAI_API_KEY="));
    }

    #[test]
    fn test_env_example_both_agents() {
        let mut config = minimal_config();
        config.agent.agent_type = AgentType::Both;
        config.auth.claude = Some(ClaudeAuthConfig {
            method: ClaudeAuthMethod::ApiKey,
        });
        config.auth.codex = Some(CodexAuthConfig {
            method: CodexAuthMethod::ApiKey,
            azure_endpoint: None,
            custom_env_key: None,
            custom_base_url: None,
        });

        let output = generate_env_example(&config);
        assert!(output.contains("ANTHROPIC_API_KEY="));
        assert!(output.contains("OPENAI_API_KEY="));
    }

    #[test]
    fn test_env_example_postgres_service() {
        let mut config = minimal_config();
        config.services.insert(
            "postgres".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("# Service credentials"));
        assert!(output.contains("POSTGRES_PASSWORD=changeme"));
    }

    #[test]
    fn test_env_example_postgres_custom_password_env() {
        let mut config = minimal_config();
        let mut extra = IndexMap::new();
        extra.insert("password_env".to_string(), toml::Value::String("MY_PG_PASS".to_string()));

        config.services.insert(
            "postgres".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra,
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("MY_PG_PASS=changeme"));
    }

    #[test]
    fn test_env_example_mysql_service() {
        let mut config = minimal_config();
        config.services.insert(
            "mysql".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("MYSQL_PASSWORD=changeme"));
        assert!(output.contains("MYSQL_ROOT_PASSWORD=changeme"));
    }

    #[test]
    fn test_env_example_mariadb_service() {
        let mut config = minimal_config();
        config.services.insert(
            "mariadb".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("MARIADB_PASSWORD=changeme"));
        assert!(output.contains("MARIADB_ROOT_PASSWORD=changeme"));
    }

    #[test]
    fn test_env_example_minio_service() {
        let mut config = minimal_config();
        config.services.insert(
            "minio".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("MINIO_ACCESS_KEY=minioadmin"));
        assert!(output.contains("MINIO_SECRET_KEY=minioadmin"));
    }

    #[test]
    fn test_env_example_disabled_service_excluded() {
        let mut config = minimal_config();
        config.services.insert(
            "postgres".to_string(),
            ServiceConfig {
                enabled: false,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let output = generate_env_example(&config);
        assert!(!output.contains("POSTGRES_PASSWORD"));
        assert!(!output.contains("# Service credentials"));
    }

    #[test]
    fn test_env_example_mcp_env_vars() {
        let mut config = minimal_config();
        config.mcp.insert(
            "github".to_string(),
            McpServerConfig {
                image: "ghcr.io/test:latest".to_string(),
                command: None,
                env: vec!["GITHUB_TOKEN".to_string(), "GH_SECRET".to_string()],
                volumes: vec![],
                port: None,
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("# MCP server credentials"));
        assert!(output.contains("GITHUB_TOKEN=your-value-here"));
        assert!(output.contains("GH_SECRET=your-value-here"));
    }

    #[test]
    fn test_env_example_unknown_service_no_credentials() {
        let mut config = minimal_config();
        config.services.insert(
            "redis".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra: IndexMap::new(),
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("# Service credentials"));
        assert!(!output.contains("REDIS_PASSWORD"));
    }

    #[test]
    fn test_env_example_mysql_custom_password_envs() {
        let mut config = minimal_config();
        let mut extra = IndexMap::new();
        extra.insert("password_env".to_string(), toml::Value::String("CUSTOM_PW".to_string()));
        extra.insert("root_password_env".to_string(), toml::Value::String("CUSTOM_ROOT_PW".to_string()));

        config.services.insert(
            "mysql".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra,
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("CUSTOM_PW=changeme"));
        assert!(output.contains("CUSTOM_ROOT_PW=changeme"));
    }

    #[test]
    fn test_env_example_mariadb_custom_password_envs() {
        let mut config = minimal_config();
        let mut extra = IndexMap::new();
        extra.insert("password_env".to_string(), toml::Value::String("MDB_PW".to_string()));
        extra.insert("root_password_env".to_string(), toml::Value::String("MDB_ROOT_PW".to_string()));

        config.services.insert(
            "mariadb".to_string(),
            ServiceConfig {
                enabled: true,
                version: None,
                port: None,
                extra,
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("MDB_PW=changeme"));
        assert!(output.contains("MDB_ROOT_PW=changeme"));
    }

    #[test]
    fn test_env_example_multiple_mcp_servers() {
        let mut config = minimal_config();
        config.mcp.insert(
            "github".to_string(),
            McpServerConfig {
                image: "test:latest".to_string(),
                command: None,
                env: vec!["GITHUB_TOKEN".to_string()],
                volumes: vec![],
                port: None,
            },
        );
        config.mcp.insert(
            "slack".to_string(),
            McpServerConfig {
                image: "test2:latest".to_string(),
                command: None,
                env: vec!["SLACK_TOKEN".to_string()],
                volumes: vec![],
                port: None,
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("GITHUB_TOKEN=your-value-here"));
        assert!(output.contains("SLACK_TOKEN=your-value-here"));
    }

    #[test]
    fn test_parse_env_key_bare() {
        assert_eq!(parse_env_key("GITHUB_TOKEN"), "GITHUB_TOKEN");
    }

    #[test]
    fn test_parse_env_key_with_substitution() {
        assert_eq!(parse_env_key("GITHUB_TOKEN=${GITHUB_TOKEN}"), "GITHUB_TOKEN");
    }

    #[test]
    fn test_parse_env_key_with_literal() {
        assert_eq!(parse_env_key("API_KEY=abc123"), "API_KEY");
    }

    #[test]
    fn test_env_example_mcp_env_key_val_format() {
        let mut config = minimal_config();
        config.mcp.insert(
            "github".to_string(),
            McpServerConfig {
                image: "ghcr.io/test:latest".to_string(),
                command: None,
                env: vec!["GITHUB_TOKEN=${GITHUB_TOKEN}".to_string()],
                volumes: vec![],
                port: None,
            },
        );

        let output = generate_env_example(&config);
        assert!(output.contains("# MCP server credentials"));
        assert!(output.contains("GITHUB_TOKEN=your-value-here"));
        // Must NOT contain the double-equals form
        assert!(!output.contains("GITHUB_TOKEN=${GITHUB_TOKEN}=your-value-here"));
    }
}
